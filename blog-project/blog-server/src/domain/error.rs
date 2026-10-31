use thiserror::Error;

#[derive(Debug, Error)]
pub enum DomainError {
    #[error("User already exists")]
    UserAlreadyExists,

    #[error("Invalid credentials")]
    InvalidCredentials,

    #[error("Post not found")]
    PostNotFound,

    #[error("Forbidden")]
    Forbidden,

    #[error("Invalid Input: {0}")]
    InvalidInput(String),

    #[error("Internal server error")]
    Internal,
}
