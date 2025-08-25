use thiserror::Error;
use crate::entity::EntityId;

pub type EcsResult<T> = Result<T, EcsError>;

#[derive(Error, Debug)]
pub enum EcsError {
    #[error("Entity {0} not found")]
    EntityNotFound(EntityId),
    
    #[error("Entity {0} is dead")]
    EntityDead(EntityId),
    
    #[error("Component type {0} not registered")]
    ComponentNotRegistered(String),
    
    #[error("Component {component} not found on entity {entity}")]
    ComponentNotFound {
        entity: EntityId,
        component: String,
    },
    
    #[error("Storage error: {0}")]
    StorageError(String),
    
    #[error("Query error: {0}")]
    QueryError(String),
    
    #[error("Serialization failed: {0}")]
    SerializationFailed(String),
    
    #[error("Deserialization failed: {0}")]
    DeserializationFailed(String),
    
    #[error("Component is still in use and cannot be removed")]
    ComponentInUse,
    
    #[error("Memory limit exceeded: {current}/{limit} bytes")]
    MemoryLimitExceeded {
        current: usize,
        limit: usize,
    },
    
    #[error("Pool exhausted for component type {0}")]
    PoolExhausted(String),
    
    #[error("Migration failed: {0}")]
    MigrationError(String),
    
    #[error("System error: {0}")]
    SystemError(String),
    
    #[error("Message error: {0}")]
    MessageError(String),
    
    #[error("Lock poisoned: {0}")]
    LockPoisoned(String),
    
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
    
    #[error("Other error: {0}")]
    Other(String),
}

impl EcsError {
    pub fn is_recoverable(&self) -> bool {
        match self {
            Self::EntityNotFound(_) | 
            Self::EntityDead(_) |
            Self::ComponentNotFound { .. } => true,
            Self::LockPoisoned(_) |
            Self::MemoryLimitExceeded { .. } |
            Self::PoolExhausted(_) => false,
            _ => true,
        }
    }
    
    pub fn should_retry(&self) -> bool {
        match self {
            Self::LockPoisoned(_) => true,
            Self::SystemError(_) => true,
            _ => false,
        }
    }
}