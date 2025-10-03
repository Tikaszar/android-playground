//! Error types for the ECS module

use thiserror::Error;

#[derive(Debug, Error)]
pub enum EcsError {
    #[error("Entity not found: {0}")]
    EntityNotFound(String),

    #[error("Component not found: {0}")]
    ComponentNotFound(String),

    #[error("System not found: {0}")]
    SystemNotFound(String),

    #[error("Query not found: {0}")]
    QueryNotFound(String),

    #[error("Storage not found: {0}")]
    StorageNotFound(String),

    #[error("Invalid entity generation")]
    InvalidGeneration,

    #[error("World locked")]
    WorldLocked,

    #[error("Not implemented: {0}")]
    NotImplemented(String),

    #[error("Operation failed: {0}")]
    OperationFailed(String),

    #[error("IO error: {0}")]
    IoError(String),

    #[error("Serialization error: {0}")]
    SerializationError(String),
}

pub type EcsResult<T> = Result<T, EcsError>;