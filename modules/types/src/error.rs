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

    #[error("Binding failed: {0}")]
    BindingFailed(String),

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