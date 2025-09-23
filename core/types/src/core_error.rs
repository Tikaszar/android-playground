//! Concrete error type for core operations (NO dyn!)

use thiserror::Error;

/// Entity ID for error reporting (temporary until we can import from core/ecs)
/// This is a simple wrapper to avoid circular dependencies
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct EntityIdError(pub u32);

impl std::fmt::Display for EntityIdError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Entity({})", self.0)
    }
}

/// Component ID for error reporting
pub type ComponentIdError = u32;

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
    
    #[error("Not initialized")]
    NotInitialized,
    
    #[error("Already initialized")]
    AlreadyInitialized,
    
    #[error("Not registered: {0}")]
    NotRegistered(String),
    
    #[error("Send error")]
    SendError,
    
    #[error("Receive error")]
    ReceiveError,
    
    #[error("Unexpected response")]
    UnexpectedResponse,
    
    #[error("Entity {0} not found")]
    EntityNotFound(EntityIdError),

    #[error("Entity {0} has expired (generation mismatch)")]
    ExpiredEntity(EntityIdError),

    #[error("Invalid entity reference")]
    InvalidEntity,

    #[error("Generation mismatch for entity {0}")]
    GenerationMismatch(EntityIdError),

    #[error("Component {1} not found on entity {0}")]
    ComponentNotFound(EntityIdError, ComponentIdError),
    
    #[error("Component type '{0}' not registered")]
    ComponentNotRegistered(String),
    
    #[error("Storage error: {0}")]
    StorageError(String),
    
    #[error("Serialization error: {0}")]
    SerializationError(String),
    
    #[error("Deserialization error: {0}")]
    DeserializationError(String),
    
    #[error("Query failed: {0}")]
    QueryFailed(String),
    
    #[error("System error: {0}")]
    SystemError(String),
    
    #[error("Message error: {0}")]
    MessageError(String),
    
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