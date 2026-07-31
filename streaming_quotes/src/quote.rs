use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

use rand::RngExt;

use crate::LibraryError;

#[derive(Debug, Clone, PartialEq)]
pub struct StockQuote {
    pub ticker: String,
    pub price: f64,
    pub volume: u64,
    pub timestamp_ms: u64,
}

impl StockQuote {
    pub fn new(ticker: String, price: f64, volume: u64, timestamp_ms: u64) -> Self {
        Self {
            ticker,
            price,
            volume,
            timestamp_ms,
        }
    }

    pub fn generate(ticker: String, price: f64) -> Result<Self, LibraryError> {
        let mut rng = rand::rng();

        let volume: u64 = match ticker.as_str() {
            "AAPL" | "MSFT" | "TSLA" => rng.random_range(1000..6000),

            "GOOG" | "AMZN" => rng.random_range(2000..4000),

            _ => rng.random_range(100..1100),
        };

        let timestamp_ms_u128 = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|err| LibraryError::SystemTimeError(err.to_string()))?
            .as_millis();

        let timestamp_ms = u64::try_from(timestamp_ms_u128)
            .map_err(|err| LibraryError::SystemTimeError(err.to_string()))?;

        Ok(Self {
            ticker,
            price,
            volume,
            timestamp_ms,
        })
    }

    pub fn to_line(&self) -> String {
        format!(
            "{}|{:.2}|{}|{}\n",
            self.ticker, self.price, self.volume, self.timestamp_ms,
        )
    }

    pub fn from_line(input: &str) -> Result<Self, LibraryError> {
        let parts: Vec<&str> = input.trim().split('|').collect();

        if parts.len() != 4 {
            return Err(LibraryError::ParseError(format!(
                "expected 4 fields, received {}",
                parts.len()
            )));
        }

        let ticker = parts[0].trim();

        if ticker.is_empty() {
            return Err(LibraryError::ParseError("ticker is empty".to_string()));
        }

        let price = parts[1]
            .parse::<f64>()
            .map_err(|err| LibraryError::ParseError(format!("invalid price: {err}")))?;

        let volume = parts[2]
            .parse::<u64>()
            .map_err(|err| LibraryError::ParseError(format!("invalid volume: {err}")))?;

        let timestamp_ms = parts[3]
            .parse::<u64>()
            .map_err(|err| LibraryError::ParseError(format!("invalid timestamp: {err}")))?;

        Ok(Self {
            ticker: ticker.to_string(),
            price,
            volume,
            timestamp_ms,
        })
    }
}

pub struct QuotePriceGenerator {
    prices: HashMap<String, f64>,
}

impl QuotePriceGenerator {
    pub fn new() -> Self {
        let mut prices = HashMap::new();

        prices.insert("AAPL".to_string(), 200.0);
        prices.insert("MSFT".to_string(), 400.0);
        prices.insert("TSLA".to_string(), 300.0);
        prices.insert("GOOG".to_string(), 180.0);
        prices.insert("AMZN".to_string(), 220.0);

        Self { prices }
    }

    pub fn generate_price(&mut self, ticker: &str) -> f64 {
        let mut rng = rand::rng();

        let price_change: f64 = rng.random_range(-2.0..2.0);

        let old_price = self.prices.get(ticker).copied().unwrap_or(100.0);

        let new_price = (old_price + price_change).max(0.01);

        self.prices.insert(ticker.to_string(), new_price);

        new_price
    }
}

impl Default for QuotePriceGenerator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn quote_round_trip() {
        let original = StockQuote::new("AAPL".to_string(), 201.25, 1500, 123456789);

        let line = original.to_line();

        let parsed = StockQuote::from_line(&line).unwrap();

        assert_eq!(original, parsed);
    }

    #[test]
    fn invalid_quote_returns_error() {
        let result = StockQuote::from_line("AAPL|wrong|100");

        assert!(matches!(result, Err(LibraryError::ParseError(_))));
    }
}
