use std::error::Error;
use std::fmt;

#[derive(Debug)]
pub enum LibraryError {
    InputOutputError(std::io::Error),
    ParseError(String),
    FormatError(String),
    UnknownError(String),
}

impl fmt::Display for LibraryError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InputOutputError(msg) => write!(f, "Invalid input: {msg}"),
            Self::ParseError(msg) => write!(f, "Parse error: {msg}"),
            Self::FormatError(msg) => write!(f, "Format error: {msg}"),
            Self::UnknownError(msg) => write!(f, "Unknown error: {msg}"),
        }
    }
}

impl Error for LibraryError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            LibraryError::InputOutputError(err) => Some(err),
            _ => None,
        }
    }
}

impl From<std::io::Error> for LibraryError {
    fn from(err: std::io::Error) -> Self {
        LibraryError::InputOutputError(err)
    }
}
