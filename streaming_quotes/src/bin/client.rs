use std::env;
use std::io::{BufRead, BufReader, ErrorKind, Read, Write};
use std::net::{SocketAddr, TcpStream, UdpSocket};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use streaming_quotes::quote::StockQuote;
use streaming_quotes::tickers::read_tickers;

const PING_INTERVAL: Duration = Duration::from_secs(2);

fn main() {
    if let Err(err) = run() {
        eprintln!("Client error: {err}");
    }
}

fn run() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();

    if args.len() != 4 {
        return Err("Usage: cargo run --bin client -- \
             <tcp-address> <udp-port> \
             <tickers-file>"
            .into());
    }

    let tcp_server_address = &args[1];

    let udp_port: u16 = args[2]
        .parse()
        .map_err(|err| format!("Invalid UDP port `{}`: {err}", args[2]))?;

    let tickers = read_tickers(&args[3])?;

    let tickers_text = tickers.join(",");

    let udp_socket = UdpSocket::bind(format!("0.0.0.0:{udp_port}"))?;

    udp_socket.set_read_timeout(Some(Duration::from_millis(500)))?;

    let mut tcp_stream = TcpStream::connect(tcp_server_address)?;

    let mut tcp_reader = BufReader::new(tcp_stream.try_clone()?);

    let mut response = String::new();

    if tcp_reader.read_line(&mut response)? == 0 {
        return Err("Server closed TCP connection".into());
    }

    print!("{response}");

    let local_ip = tcp_stream.local_addr()?.ip();

    let advertised_udp_address = SocketAddr::new(local_ip, udp_port);

    let command = format!("STREAM {} {}\n", advertised_udp_address, tickers_text,);

    tcp_stream.write_all(command.as_bytes())?;

    tcp_stream.flush()?;

    response.clear();

    if tcp_reader.read_line(&mut response)? == 0 {
        return Err("Server closed TCP connection before \
             response"
            .into());
    }

    if response.trim() != "OK" {
        return Err(format!("Server response: {}", response.trim()).into());
    }

    println!("Subscription accepted");

    let running = Arc::new(AtomicBool::new(true));

    let server_udp_address: Arc<Mutex<Option<SocketAddr>>> = Arc::new(Mutex::new(None));

    {
        let mut monitor_stream = tcp_stream.try_clone()?;

        let monitor_running = Arc::clone(&running);

        thread::spawn(move || {
            let mut buffer = [0_u8; 1];

            match monitor_stream.read(&mut buffer) {
                Ok(0) => {
                    println!("TCP connection closed");
                }

                Ok(_) => {}

                Err(err) => {
                    eprintln!("TCP monitor error: {err}");
                }
            }

            monitor_running.store(false, Ordering::Relaxed);
        });
    }

    let ping_handle = {
        let ping_socket = udp_socket.try_clone()?;

        let ping_running = Arc::clone(&running);

        let ping_address = Arc::clone(&server_udp_address);

        thread::spawn(move || {
            while ping_running.load(Ordering::Relaxed) {
                thread::sleep(PING_INTERVAL);

                if !ping_running.load(Ordering::Relaxed) {
                    break;
                }

                let destination = match ping_address.lock() {
                    Ok(address) => *address,

                    Err(err) => {
                        eprintln!(
                            "PING lock error: \
                                 {err}"
                        );

                        ping_running.store(false, Ordering::Relaxed);

                        break;
                    }
                };

                let Some(destination) = destination else {
                    continue;
                };

                if let Err(err) = ping_socket.send_to(b"PING\n", destination) {
                    eprintln!(
                        "Failed to send PING: \
                         {err}"
                    );

                    ping_running.store(false, Ordering::Relaxed);

                    break;
                }
            }
        })
    };

    let mut buffer = [0_u8; 2048];

    while running.load(Ordering::Relaxed) {
        match udp_socket.recv_from(&mut buffer) {
            Ok((received_bytes, sender)) => {
                let mut address = server_udp_address.lock().map_err(|err| {
                    format!(
                        "Address lock \
                                 error: {err}"
                    )
                })?;

                *address = Some(sender);

                drop(address);

                let message = String::from_utf8_lossy(&buffer[..received_bytes]);

                match StockQuote::from_line(&message) {
                    Ok(quote) => {
                        println!(
                            "{} price={:.2} \
                             volume={} \
                             timestamp_ms={}",
                            quote.ticker, quote.price, quote.volume, quote.timestamp_ms,
                        );
                    }

                    Err(err) => {
                        eprintln!("Invalid quote: {err}");
                    }
                }
            }

            Err(err)
                if err.kind() == ErrorKind::WouldBlock || err.kind() == ErrorKind::TimedOut =>
            {
                continue;
            }

            Err(err) => {
                eprintln!("UDP receive error: {err}");

                running.store(false, Ordering::Relaxed);

                break;
            }
        }
    }

    running.store(false, Ordering::Relaxed);

    let _ = ping_handle.join();

    println!("Client stopped");

    Ok(())
}
