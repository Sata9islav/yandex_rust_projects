use crate::LibraryError;

/// Поддерживаемые форматы данных.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Format {
    Csv,
    Text,
    Bin,
}

impl Format {
    /// Преобразует строковое имя формата в Format.
    /// Возвращает ошибку для неизвестного формата.
    pub fn parse(value: &str) -> Result<Self, LibraryError> {
        match value.to_lowercase().as_str() {
            "csv" => Ok(Self::Csv),
            "text" => Ok(Self::Text),
            "bin" => Ok(Self::Bin),
            unknown => Err(LibraryError::FormatError(format!(
                "Unknown format: {unknown}"
            ))),
        }
    }
}
