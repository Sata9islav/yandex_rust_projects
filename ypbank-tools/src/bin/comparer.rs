use std::env;
use std::fs::File;

use ypbank_tools::{Format, LibraryError, compare_transactions, read_transactions};

fn run() -> Result<(), LibraryError> {
    let args: Vec<String> = env::args().collect();

    if args.len() != 5 {
        return Err(LibraryError::FormatError(
            "Usage: comparer <first-file> <first-format> <second-file> <second-format>".to_string(),
        ));
    }

    let first_path = &args[1];
    let first_fromat = Format::parse(&args[2])?;

    let second_path = &args[3];
    let second_format = Format::parse(&args[4])?;

    let first_file = File::open(first_path)?;
    let second_file = File::open(second_path)?;

    let first_transactions = read_transactions(first_file, first_fromat)?;
    let second_transactions = read_transactions(second_file, second_format)?;

    match compare_transactions(&first_transactions, &second_transactions) {
        None => {
            println!("Transaction equal");
        }

        Some(difference) => {
            println!("{difference}");
        }
    }

    Ok(())
}

fn main() {
    if let Err(err) = run() {
        eprintln!("Error: {err}");
        std::process::exit(1);
    }
}
