use actix_web::{HttpResponse, ResponseError, http::StatusCode};

use serde::Serialize;

use crate::domain::error::DomainError;

#[derive(Debug)]
pub struct ApiError(pub DomainError);

#[derive(Serialize)]
struct ErrorResponse {
    error: String,
}

impl std::fmt::Display for ApiError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl ResponseError for ApiError {
    fn status_code(&self) -> StatusCode {
        match &self.0 {
            DomainError::PostNotFound => StatusCode::NOT_FOUND,
            DomainError::UserAlreadyExists => StatusCode::CONFLICT,
            DomainError::InvalidCredentials => StatusCode::UNAUTHORIZED,
            DomainError::Forbidden => StatusCode::FORBIDDEN,
            DomainError::InvalidInput(_) => StatusCode::BAD_REQUEST,
            DomainError::Internal => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    fn error_response(&self) -> HttpResponse {
        let message = match &self.0 {
            DomainError::Internal => "Internal server error".to_string(),
            error => error.to_string(),
        };

        HttpResponse::build(self.status_code()).json(ErrorResponse { error: message })
    }
}

impl From<DomainError> for ApiError {
    fn from(error: DomainError) -> Self {
        Self(error)
    }
}
