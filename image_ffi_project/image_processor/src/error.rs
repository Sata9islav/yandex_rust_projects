use std::path::PathBuf;

#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("file not found: {}", .0.display())]
    FileNotFound(PathBuf),
    #[error("failed to read the parameters file {}: {source}", path.display())]
    ReadParams { path: PathBuf, #[source] source: std::io::Error },
    #[error("failed to open the image {}: {source}", path.display())]
    OpenImage { path: PathBuf, #[source] source: image::ImageError },
    #[error("failed to save PNG {}: {source}", path.display())]
    SaveImage { path: PathBuf, #[source] source: image::ImageError },
    #[error("invalid plugin name: {0} (only ASCII-letters, numbers and _)")]
    InvalidPluginName(String),
    #[error("failed to load library {}: {source}", path.display())]
    LoadPlugin { path: PathBuf, #[source] source: libloading::Error },
    #[error("In lybrary {} not found process_image: {source}", path.display())]
    MissingSymbol { path: PathBuf, #[source] source: libloading::Error },
    #[error("parameters contain a null byte (NUL): {0}")]
    InvalidParams(#[from] std::ffi::NulError),
    #[error("size RGBA-buffer does not match the image dimensions")]
    InvalidBuffer,
}

pub type Result<T> = std::result::Result<T, AppError>;
