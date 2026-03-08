use thiserror::Error;

#[derive(Error,Debug)]
pub enum OxideError{
    #[error("Configuration error: {0}")]
    Config(String),

    #[error("Database error: {0}")]
    Database(String),

    #[error("Build Error: {0}")]
    Build(String),

    #[error("Runtime Error: {0}")]
    Runtime(String),

    #[error("Proxy error: {0}")]
    Proxy(String),

    #[error("Internal error: {0}")]
    Internal(String),
}
