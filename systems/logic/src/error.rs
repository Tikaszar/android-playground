use thiserror::Error;
use uuid::Uuid;

#[derive(Error, Debug)]
pub enum LogicError {
    #[error("Component type {0} not registered")]
    ComponentNotRegistered(String),
    
    #[error("Entity {0} not found")]
    EntityNotFound(Uuid),
    
    #[error("Archetype {0} not found")]
    ArchetypeNotFound(u64),
    
    #[error("System {0} not found")]
    SystemNotFound(String),
    
    #[error("Query cache {0} expired")]
    QueryCacheExpired(u64),
    
    #[error("Circular dependency detected in system graph")]
    CircularDependency,
    
    #[error("System {0} failed after {1} retries")]
    SystemFailure(String, u32),
    
    #[error("Component migration failed: {0}")]
    MigrationError(String),
    
    #[error("Network synchronization error: {0}")]
    NetworkError(String),
    
    #[error("Memory limit exceeded: {current} > {limit} MB")]
    MemoryLimitExceeded { current: usize, limit: usize },
    
    #[error("Hot-reload error: {0}")]
    HotReloadError(String),
    
    #[error("Event system error: {0}")]
    EventError(String),
    
    #[error("Storage error: {0}")]
    StorageError(String),
    
    #[error("Serialization error: {0}")]
    SerializationError(String),
    
    #[error("Core ECS error: {0}")]
    CoreError(#[from] playground_ecs::EcsError),
    
    #[error("Initialization failed: {0}")]
    InitializationFailed(String),
    
    #[error("System error: {0}")]
    SystemError(String),
}

pub type LogicResult<T> = Result<T, LogicError>;