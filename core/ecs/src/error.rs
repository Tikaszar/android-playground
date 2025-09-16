//! Error types for the ECS
//! 
//! These provide clear error boundaries between layers.

use thiserror::Error;
use crate::{EntityId, ComponentId};

/// Core ECS errors
#[derive(Error, Debug)]
pub enum CoreError {
    /// World not initialized
    #[error("World has not been initialized. Call initialize_world() first")]
    NotInitialized,
    
    /// World already initialized
    #[error("World has already been initialized")]
    AlreadyInitialized,
    
    /// System not registered
    #[error("System '{0}' is not registered. Ensure the system has called register()")]
    NotRegistered(String),
    
    /// Channel send failed
    #[error("Failed to send command through channel")]
    SendError,
    
    /// Channel receive failed
    #[error("Failed to receive response from channel")]
    ReceiveError,
    
    /// Unexpected response type
    #[error("Received unexpected response type")]
    UnexpectedResponse,
    
    /// Entity not found
    #[error("Entity {0} not found")]
    EntityNotFound(EntityId),
    
    /// Component not found
    #[error("Component {1:?} not found on entity {0}")]
    ComponentNotFound(EntityId, ComponentId),
    
    /// Component not registered
    #[error("Component {0} not registered")]
    ComponentNotRegistered(String),
    
    /// Storage error
    #[error("Storage error: {0}")]
    StorageError(String),
    
    /// Serialization error
    #[error("Failed to serialize: {0}")]
    SerializationError(String),
    
    /// Deserialization error
    #[error("Failed to deserialize: {0}")]
    DeserializationError(String),
    
    /// Query failed
    #[error("Query failed: {0}")]
    QueryFailed(String),
    
    /// System error
    #[error("System error: {0}")]
    SystemError(String),
    
    /// Message error
    #[error("Message error: {0}")]
    MessageError(String),
    
    /// Generic error for system-specific failures
    #[error("{0}")]
    Generic(String),
}

/// Result type for ECS operations
pub type CoreResult<T> = Result<T, CoreError>;

// Keep compatibility aliases
pub type EcsError = CoreError;
pub type EcsResult<T> = CoreResult<T>;