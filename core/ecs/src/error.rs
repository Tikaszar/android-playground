//! Error types for the ECS

use thiserror::Error;
use crate::entity::EntityId;

/// Result type for ECS operations
pub type EcsResult<T> = Result<T, EcsError>;

/// Error types for ECS operations
#[derive(Error, Debug)]
pub enum EcsError {
    #[error("Component {0} not registered")]
    ComponentNotRegistered(String),
    
    #[error("Component not found for entity {entity}: {component}")]
    ComponentNotFound {
        entity: EntityId,
        component: String,
    },
    
    #[error("Entity {0} not found")]
    EntityNotFound(EntityId),
    
    #[error("Storage error: {0}")]
    StorageError(String),
    
    #[error("Serialization failed: {0}")]
    SerializationFailed(String),
    
    #[error("Deserialization failed: {0}")]
    DeserializationFailed(String),
    
    #[error("Query failed: {0}")]
    QueryFailed(String),
    
    #[error("System error: {0}")]
    SystemError(String),
    
    #[error("Message error: {0}")]
    MessageError(String),
    
    #[error("Pool exhausted: {0}")]
    PoolExhausted(String),
    
    #[error("Generic error: {0}")]
    Generic(String),
}

impl From<anyhow::Error> for EcsError {
    fn from(err: anyhow::Error) -> Self {
        Self::Generic(err.to_string())
    }
}

impl From<std::io::Error> for EcsError {
    fn from(err: std::io::Error) -> Self {
        Self::Generic(err.to_string())
    }
}

impl From<bincode::Error> for EcsError {
    fn from(err: bincode::Error) -> Self {
        Self::SerializationFailed(err.to_string())
    }
}