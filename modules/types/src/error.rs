//! Error types for module system

use thiserror::Error;

#[derive(Error, Debug)]
pub enum ModuleError {
    #[error("Module not found: {0}")]
    NotFound(String),

    #[error("Module load failed: {0}")]
    LoadFailed(String),

    #[error("Module already loaded: {0}")]
    AlreadyLoaded(String),

    #[error("Invalid module: {0}")]
    InvalidModule(String),

    #[error("Dependency missing: {0}")]
    DependencyMissing(String),

    #[error("Feature missing: {0}")]
    FeatureMissing(String),

    #[error("Version mismatch: {0}")]
    VersionMismatch(String),

    #[error("API version mismatch: expected {expected}, found {found}")]
    ApiVersionMismatch { expected: u32, found: u32 },

    #[error("State format version mismatch: expected {expected}, found {found}")]
    StateVersionMismatch { expected: u32, found: u32 },

    #[error("Binding failed: {0}")]
    BindingFailed(String),

    #[error("State save failed: {0}")]
    StateSaveFailed(String),

    #[error("State restore failed: {0}")]
    StateRestoreFailed(String),

    #[error("State serialization failed: {0}")]
    StateSerializationFailed(String),

    #[error("Serialization error: {0}")]
    SerializationError(String),

    #[error("Deserialization error: {0}")]
    DeserializationError(String),

    #[error("Generic error: {0}")]
    Generic(String),
}

pub type ModuleResult<T> = Result<T, ModuleError>;