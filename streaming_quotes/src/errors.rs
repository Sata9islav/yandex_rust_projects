use std::error::Error;
use std::fmt;

#[derive(Debug)]
pub enum LibraryError {
    InputOutputError(std::io::Error),
    ParseError(String),
    InvalidCommandError(String),
    InvalidUdpError(String),
    EmptyTickersError(String),
    UnknownTickerError(String),
    SystemTimeError(String),
}

impl fmt::Display for LibraryError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InputOutputError(err) => {
                write!(f, "input/output error: {err}")
            }

            Self::ParseError(message) => {
                write!(f, "parse error: {message}")
            }

            Self::InvalidCommandError(command) => {
                write!(f, "invalid command: {command}")
            }

            Self::InvalidUdpError(address) => {
                write!(f, "invalid UDP address: {address}")
            }

            Self::EmptyTickersError(message) => {
                write!(f, "empty ticker list: {message}")
            }

            Self::UnknownTickerError(ticker) => {
                write!(f, "unknown ticker: {ticker}")
            }

            Self::SystemTimeError(message) => {
                write!(f, "system time error: {message}")
            }
        }
    }
}

impl Error for LibraryError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::InputOutputError(err) => Some(err),
            _ => None,
        }
    }
}

impl From<std::io::Error> for LibraryError {
    fn from(err: std::io::Error) -> Self {
        Self::InputOutputError(err)
    }
}
