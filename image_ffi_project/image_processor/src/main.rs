use std::path::PathBuf;
use clap::Parser;

#[derive(Debug, Parser)]
#[command(version, about = "Processing PNG by Rust-plugin")]
struct Cli {
    #[arg(long)]
    input: PathBuf,
    #[arg(long)]
    output: PathBuf,
    #[arg(long)]
    plugin: String,
    #[arg(long)]
    params: PathBuf,
    #[arg(long, default_value = "target/debug")]
    plugin_path: PathBuf,
}

fn main() {
    let cli = Cli::parse();
    if let Err(error) = image_processor::process_image(
        &cli.input, &cli.output, &cli.plugin, &cli.params, &cli.plugin_path,
    ) {
        eprintln!("Ошибка: {error}");
        std::process::exit(1);
    }
    println!("Готово: {}", cli.output.display());
}
