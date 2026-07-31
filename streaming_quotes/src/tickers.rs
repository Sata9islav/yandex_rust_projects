use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

use crate::LibraryError;

pub fn read_tickers<P: AsRef<Path>>(path: P) -> Result<Vec<String>, LibraryError> {
    let file = File::open(path)?;

    read_tickers_from_reader(BufReader::new(file))
}

pub fn read_tickers_from_reader<R: BufRead>(reader: R) -> Result<Vec<String>, LibraryError> {
    let mut tickers = Vec::new();

    for line_result in reader.lines() {
        let line = line_result?;
        let ticker = line.trim();

        if ticker.is_empty() {
            continue;
        }

        tickers.push(ticker.to_ascii_uppercase());
    }

    if tickers.is_empty() {
        return Err(LibraryError::EmptyTickersError(
            "ticker file contains no tickers".to_string(),
        ));
    }

    Ok(tickers)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn reads_tickers_and_skips_empty_lines() {
        let input = Cursor::new("AAPL\n\n MSFT \nTSLA\n");

        let tickers = read_tickers_from_reader(input).unwrap();

        assert_eq!(tickers, vec!["AAPL", "MSFT", "TSLA"]);
    }

    #[test]
    fn empty_ticker_list_returns_error() {
        let input = Cursor::new("\n   \n");

        let result = read_tickers_from_reader(input);

        assert!(matches!(result, Err(LibraryError::EmptyTickersError(_))));
    }
}
