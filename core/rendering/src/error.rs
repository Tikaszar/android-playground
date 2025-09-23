//! Rendering error types

use thiserror::Error;

#[derive(Error, Debug)]
pub enum RenderError {
    #[error("Renderer not initialized")]
    NotInitialized,

    #[error("Renderer already initialized")]
    AlreadyInitialized,

    #[error("Invalid resource ID: {0}")]
    InvalidResource(u32),

    #[error("Resource creation failed: {0}")]
    ResourceCreationFailed(String),

    #[error("Invalid operation: {0}")]
    InvalidOperation(String),

    #[error("Backend not available: {0}")]
    BackendNotAvailable(String),

    #[error("Feature not supported: {0}")]
    FeatureNotSupported(String),

    #[error("Shader compilation failed: {0}")]
    ShaderCompilationFailed(String),

    #[error("Command execution failed: {0}")]
    CommandExecutionFailed(String),

    #[error("Context lost")]
    ContextLost,

    #[error("Out of memory")]
    OutOfMemory,

    #[error("Serialization error: {0}")]
    SerializationError(String),

    #[error("VTable error: {0}")]
    VTableError(String),

    #[error("Generic error: {0}")]
    Generic(String),
}

pub type RenderResult<T> = Result<T, RenderError>;