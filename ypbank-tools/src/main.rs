use std::env;
use std::fs::File;
use std::io::{self};

use ypbank_tools::{Format, LibraryError, convert};

fn run() -> Result<(), LibraryError> {
    let args: Vec<String> = env::args().collect();

    if args.len() != 4 {
        return Err(LibraryError::FormatError(
            "Usage: ypbank-tools <input-file|-> <input-format> <output-format>".to_string(),
        ));
    }

    let input_path = &args[1];
    let input_format = Format::parse(&args[2])?;
    let output_format = Format::parse(&args[3])?;

    let stdout = io::stdout();
    let output = stdout.lock();

    if input_path == "-" {
        let stdin = io::stdin();
        let input = stdin.lock();

        convert(input, output, input_format, output_format)?;
    } else {
        let input = File::open(input_path)?;
        convert(input, output, input_format, output_format)?;
    }

    Ok(())
}

fn main() {
    if let Err(err) = run() {
        eprintln!("Error: {err}");
        std::process::exit(1);
    }
}
