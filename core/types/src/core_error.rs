//! Concrete error type for core operations (NO dyn!)

use thiserror::Error;

/// Concrete error type for all core operations
#[derive(Debug, Error, Clone)]
pub enum CoreError {
    #[error("IO error: {0}")]
    Io(String),
    
    #[error("Network error: {0}")]
    Network(String),
    
    #[error("Parse error: {0}")]
    Parse(String),
    
    #[error("Not found: {0}")]
    NotFound(String),
    
    #[error("Already exists: {0}")]
    AlreadyExists(String),
    
    #[error("Invalid state: {0}")]
    InvalidState(String),
    
    #[error("Invalid input: {0}")]
    InvalidInput(String),
    
    #[error("Not initialized: {0}")]
    NotInitialized(String),
    
    #[error("Permission denied: {0}")]
    PermissionDenied(String),
    
    #[error("Timeout: {0}")]
    Timeout(String),
    
    #[error("Cancelled: {0}")]
    Cancelled(String),
    
    #[error("Generic error: {0}")]
    Generic(String),
}

impl From<std::io::Error> for CoreError {
    fn from(err: std::io::Error) -> Self {
        CoreError::Io(err.to_string())
    }
}

impl From<serde_json::Error> for CoreError {
    fn from(err: serde_json::Error) -> Self {
        CoreError::Parse(err.to_string())
    }
}

impl From<tokio::time::error::Elapsed> for CoreError {
    fn from(err: tokio::time::error::Elapsed) -> Self {
        CoreError::Timeout(err.to_string())
    }
}

/// Result type using CoreError
pub type CoreResult<T> = Result<T, CoreError>;