mod cli;
mod commands;
mod token;

use blog_client::{BlogClient, Transport};
use clap::Parser;

use cli::Cli;

#[tokio::main]
async fn main() {
    if let Err(error) = run().await {
        eprintln!("Ошибка: {error}");
        std::process::exit(1);
    }
}

async fn run() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    let server = cli.server.unwrap_or_else(|| {
        if cli.grpc {
            "http://127.0.0.1:50051".to_string()
        } else {
            "http://127.0.0.1:8080".to_string()
        }
    });

    let transport = if cli.grpc {
        Transport::Grpc(server)
    } else {
        Transport::Http(server)
    };

    let mut client = BlogClient::new(transport).await?;

    if let Some(saved_token) = token::load_token()? {
        client.set_token(saved_token);
    }

    commands::execute(&mut client, cli.command).await?;

    Ok(())
}
