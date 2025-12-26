pub mod error {
    use thiserror::Error;
    #[derive(Debug, Error)]
    pub enum InternalError {
        #[error("Failed to acquire lock")]
        LockFailed,
        #[error("Endpoint not found: {0}")]
        EndpointNotFound(String),
        #[error("Failed to initialize Logger")]
        LoggerInitError,
        #[error("Failed to parse command input")]
        ParserError,
        #[error("IO error: {0}")]
        Io(#[from] std::io::Error),
    }

    impl From<InternalError> for std::io::Error {
        fn from(value: InternalError) -> Self {
            std::io::Error::other(value.to_string())
        }
    }
}

pub mod result {
    use crate::util::error::InternalError;

    pub type InternalResult<T> = Result<T, InternalError>;
}
