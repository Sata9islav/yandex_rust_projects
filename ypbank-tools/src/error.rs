use std::error::Error;
use std::fmt;

/// Ошибки библиотеки обработки транзакций.
#[derive(Debug)]
pub enum LibraryError {
    /// Ошибка ввода-вывода.
    InputOutputError(std::io::Error),
    /// Ошибка разбора данных.
    ParseError(String),
    /// Ошибка формата данных.
    FormatError(String),
}

impl fmt::Display for LibraryError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InputOutputError(msg) => write!(f, "Invalid input: {msg}"),
            Self::ParseError(msg) => write!(f, "Parse error: {msg}"),
            Self::FormatError(msg) => write!(f, "Format error: {msg}"),
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
