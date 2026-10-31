use thiserror::Error;

#[derive(Debug, Error)]
pub enum BlogClientError {
    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),

    #[error("gRPC error: {0}")]
    Grpc(tonic::Status),

    #[error("gRPC transport error: {0}")]
    GrpcTransport(#[from] tonic::transport::Error),

    #[error("Resource not found")]
    NotFound,

    #[error("Unathorized")]
    Unathorized,

    #[error("Invalid request: {0}")]
    InvalidRequest(String),

    #[error("Unexpected response: {0}")]
    UnexpectedResponse(String),

    #[error("Forbidden")]
    Forbidden,
}

impl From<tonic::Status> for BlogClientError {
    fn from(status: tonic::Status) -> Self {
        match status.code() {
            tonic::Code::NotFound => Self::NotFound,
            tonic::Code::Unauthenticated => Self::Unathorized,
            tonic::Code::PermissionDenied => Self::Forbidden,
            tonic::Code::InvalidArgument | tonic::Code::AlreadyExists => {
                Self::InvalidRequest(status.message().to_string())
            }
            _ => Self::Grpc(status),
        }
    }
}
