use std::collections::HashSet;
use std::net::TcpListener;
use std::sync::Arc;
use std::thread;
use std::time::Duration;

use streaming_quotes::protocol::{create_subscribers, handle_client, publish_quote};
use streaming_quotes::quote::{QuotePriceGenerator, StockQuote};
use streaming_quotes::tickers::read_tickers;

fn main() {
    if let Err(err) = run() {
        eprintln!("Server error: {err}");
    }
}

fn run() -> Result<(), Box<dyn std::error::Error>> {
    let tickers = read_tickers("assets/tickers.txt")?;

    let allowed_tickers: HashSet<String> = tickers.iter().cloned().collect();

    let allowed_tickers = Arc::new(allowed_tickers);

    let subscribers = create_subscribers();

    /*
     * Общий генератор создаёт котировки
     * для всех известных тикеров.
     */
    {
        let generator_tickers = tickers.clone();
        let generator_subscribers = Arc::clone(&subscribers);

        thread::spawn(move || {
            let mut price_generator = QuotePriceGenerator::new();

            loop {
                for ticker in &generator_tickers {
                    let price = price_generator.generate_price(ticker);

                    let quote = match StockQuote::generate(ticker.clone(), price) {
                        Ok(quote) => quote,

                        Err(err) => {
                            eprintln!(
                                "Quote generation \
                                     error: {err}"
                            );

                            continue;
                        }
                    };

                    publish_quote(&generator_subscribers, quote);
                }

                thread::sleep(Duration::from_millis(500));
            }
        });
    }

    let listener = TcpListener::bind("127.0.0.1:7878")?;

    println!("Server listening on 127.0.0.1:7878");

    for stream_result in listener.incoming() {
        match stream_result {
            Ok(stream) => {
                let client_tickers = Arc::clone(&allowed_tickers);

                let client_subscribers = Arc::clone(&subscribers);

                thread::spawn(move || {
                    if let Err(err) = handle_client(stream, client_tickers, client_subscribers) {
                        eprintln!(
                            "Client handler error: \
                             {err}"
                        );
                    }
                });
            }

            Err(err) => {
                eprintln!("Connection error: {err}");
            }
        }
    }

    Ok(())
}
