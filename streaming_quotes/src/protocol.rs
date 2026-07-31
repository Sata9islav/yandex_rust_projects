use std::collections::HashSet;
use std::io::{BufRead, BufReader, ErrorKind, Write};
use std::net::{SocketAddr, TcpStream, UdpSocket};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{self, Receiver, RecvTimeoutError, Sender};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

use crate::LibraryError;
use crate::quote::StockQuote;

const PING_TIMEOUT: Duration = Duration::from_secs(6);

const STREAM_CHECK_INTERVAL: Duration = Duration::from_millis(200);

#[derive(Debug)]
pub struct StreamCommand {
    pub udp_address: SocketAddr,
    pub tickers: Vec<String>,
}

#[derive(Debug)]
pub enum Command {
    Stream(StreamCommand),
    Ping,
}

pub type Subscribers = Arc<Mutex<Vec<Sender<StockQuote>>>>;

pub fn create_subscribers() -> Subscribers {
    Arc::new(Mutex::new(Vec::new()))
}

pub fn add_subscriber(subscribers: &Subscribers) -> Result<Receiver<StockQuote>, LibraryError> {
    let (sender, receiver) = mpsc::channel::<StockQuote>();

    let mut senders = subscribers
        .lock()
        .map_err(|err| LibraryError::ParseError(format!("subscriber lock error: {err}")))?;

    senders.push(sender);

    Ok(receiver)
}

pub fn publish_quote(subscribers: &Subscribers, quote: StockQuote) {
    let mut senders = match subscribers.lock() {
        Ok(senders) => senders,

        Err(err) => {
            eprintln!("Failed to lock subscribers: {err}");
            return;
        }
    };

    senders.retain(|sender| sender.send(quote.clone()).is_ok());
}

pub fn process_command(
    input: &str,
    allowed_tickers: &HashSet<String>,
) -> Result<Command, LibraryError> {
    let mut parts = input.split_whitespace();

    let command = parts
        .next()
        .ok_or_else(|| LibraryError::InvalidCommandError("command is empty".to_string()))?;

    match command {
        "PING" => {
            if parts.next().is_some() {
                return Err(LibraryError::InvalidCommandError(input.to_string()));
            }

            Ok(Command::Ping)
        }

        "STREAM" => {
            let address_text = parts.next().ok_or_else(|| {
                LibraryError::InvalidUdpError("UDP address is missing".to_string())
            })?;

            let udp_address = address_text
                .parse::<SocketAddr>()
                .map_err(|err| LibraryError::InvalidUdpError(format!("{address_text}: {err}")))?;

            let tickers_text = parts.next().ok_or_else(|| {
                LibraryError::EmptyTickersError("ticker list is missing".to_string())
            })?;

            if parts.next().is_some() {
                return Err(LibraryError::InvalidCommandError(input.to_string()));
            }

            let tickers: Vec<String> = tickers_text
                .split(',')
                .map(str::trim)
                .filter(|ticker| !ticker.is_empty())
                .map(|ticker| ticker.to_ascii_uppercase())
                .collect();

            if tickers.is_empty() {
                return Err(LibraryError::EmptyTickersError(
                    "ticker list is empty".to_string(),
                ));
            }

            for ticker in &tickers {
                if !allowed_tickers.contains(ticker) {
                    return Err(LibraryError::UnknownTickerError(ticker.clone()));
                }
            }

            Ok(Command::Stream(StreamCommand {
                udp_address,
                tickers,
            }))
        }

        _ => Err(LibraryError::InvalidCommandError(command.to_string())),
    }
}

pub fn start_udp_stream(
    udp_address: SocketAddr,
    tickers: Vec<String>,
    receiver: Receiver<StockQuote>,
    active: Arc<AtomicBool>,
) -> Result<thread::JoinHandle<()>, LibraryError> {
    let socket = UdpSocket::bind("0.0.0.0:0")?;

    socket.connect(udp_address)?;

    socket.set_nonblocking(true)?;

    let ticker_filter: HashSet<String> = tickers.into_iter().collect();

    let handle = thread::spawn(move || {
        let mut last_ping = Instant::now();
        let mut ping_buffer = [0_u8; 128];

        while active.load(Ordering::Relaxed) {
            /*
             * Принимаем UDP PING.
             *
             * Сокет неблокирующий, поэтому при отсутствии
             * сообщения recv() вернёт WouldBlock.
             */
            match socket.recv(&mut ping_buffer) {
                Ok(received_bytes) => {
                    let message = String::from_utf8_lossy(&ping_buffer[..received_bytes]);

                    if message.trim().eq_ignore_ascii_case("PING") {
                        last_ping = Instant::now();
                    }
                }

                Err(err) if err.kind() == ErrorKind::WouldBlock => {}

                Err(err) => {
                    eprintln!("UDP receive error: {err}");

                    active.store(false, Ordering::Relaxed);

                    break;
                }
            }

            if last_ping.elapsed() > PING_TIMEOUT {
                eprintln!("PING timeout for {udp_address}");

                active.store(false, Ordering::Relaxed);

                break;
            }

            /*
             * Получаем котировку от общего генератора.
             */
            match receiver.recv_timeout(STREAM_CHECK_INTERVAL) {
                Ok(quote) => {
                    if !ticker_filter.contains(&quote.ticker) {
                        continue;
                    }

                    let message = quote.to_line();

                    if let Err(err) = socket.send(message.as_bytes()) {
                        eprintln!(
                            "Failed to send quote to \
                             {udp_address}: {err}"
                        );

                        active.store(false, Ordering::Relaxed);

                        break;
                    }
                }

                Err(RecvTimeoutError::Timeout) => {}

                Err(RecvTimeoutError::Disconnected) => {
                    active.store(false, Ordering::Relaxed);

                    break;
                }
            }
        }

        println!("UDP stream stopped for {udp_address}");
    });

    Ok(handle)
}

pub fn handle_client(
    stream: TcpStream,
    allowed_tickers: Arc<HashSet<String>>,
    subscribers: Subscribers,
) -> Result<(), LibraryError> {
    let mut writer = stream.try_clone()?;
    let mut reader = BufReader::new(stream.try_clone()?);

    writer.write_all(b"Welcome to Streaming\n")?;

    writer.flush()?;

    let mut line = String::new();

    let received = reader.read_line(&mut line)?;

    if received == 0 {
        return Ok(());
    }

    let command = match process_command(line.trim(), &allowed_tickers) {
        Ok(command) => command,

        Err(err) => {
            let response = format!("ERR {err}\n");

            writer.write_all(response.as_bytes())?;

            writer.flush()?;

            return Ok(());
        }
    };

    match command {
        Command::Ping => {
            writer.write_all(b"PONG\n")?;
            writer.flush()?;

            Ok(())
        }

        Command::Stream(stream_command) => {
            let receiver = add_subscriber(&subscribers)?;

            writer.write_all(b"OK\n")?;
            writer.flush()?;

            let active = Arc::new(AtomicBool::new(true));

            let stream_handle = start_udp_stream(
                stream_command.udp_address,
                stream_command.tickers,
                receiver,
                Arc::clone(&active),
            )?;

            /*
             * Периодически проверяем TCP-соединение.
             *
             * Если клиент закрыл TCP, read_line вернёт 0.
             */
            stream.set_read_timeout(Some(Duration::from_millis(500)))?;

            loop {
                if !active.load(Ordering::Relaxed) {
                    break;
                }

                line.clear();

                match reader.read_line(&mut line) {
                    Ok(0) => {
                        active.store(false, Ordering::Relaxed);

                        break;
                    }

                    Ok(_) => {
                        if line.trim().eq_ignore_ascii_case("EXIT") {
                            active.store(false, Ordering::Relaxed);

                            break;
                        }
                    }

                    Err(err)
                        if err.kind() == ErrorKind::WouldBlock
                            || err.kind() == ErrorKind::TimedOut =>
                    {
                        continue;
                    }

                    Err(err) => {
                        eprintln!("TCP client error: {err}");

                        active.store(false, Ordering::Relaxed);

                        break;
                    }
                }
            }

            let _ = stream_handle.join();

            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn allowed() -> HashSet<String> {
        ["AAPL", "MSFT", "TSLA"]
            .into_iter()
            .map(str::to_string)
            .collect()
    }

    #[test]
    fn parses_stream_command() {
        let command = process_command("STREAM 127.0.0.1:9000 AAPL,TSLA", &allowed()).unwrap();

        match command {
            Command::Stream(stream) => {
                assert_eq!(stream.udp_address, "127.0.0.1:9000".parse().unwrap());

                assert_eq!(stream.tickers, vec!["AAPL", "TSLA"]);
            }

            _ => panic!("Expected STREAM"),
        }
    }

    #[test]
    fn rejects_invalid_command() {
        let result = process_command("WRONG", &allowed());

        assert!(matches!(result, Err(LibraryError::InvalidCommandError(_))));
    }

    #[test]
    fn rejects_invalid_udp_address() {
        let result = process_command("STREAM invalid-address AAPL", &allowed());

        assert!(matches!(result, Err(LibraryError::InvalidUdpError(_))));
    }

    #[test]
    fn rejects_empty_tickers() {
        let result = process_command("STREAM 127.0.0.1:9000 ,,,", &allowed());

        assert!(matches!(result, Err(LibraryError::EmptyTickersError(_))));
    }

    #[test]
    fn rejects_unknown_ticker() {
        let result = process_command("STREAM 127.0.0.1:9000 UNKNOWN", &allowed());

        assert!(matches!(result, Err(LibraryError::UnknownTickerError(_))));
    }

    #[test]
    fn ticker_filter_works() {
        let filter: HashSet<String> = ["AAPL", "TSLA"].into_iter().map(str::to_string).collect();

        assert!(filter.contains("AAPL"));
        assert!(!filter.contains("MSFT"));
    }
}
