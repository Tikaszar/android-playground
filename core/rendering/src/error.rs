use thiserror::Error;

#[derive(Error, Debug)]
pub enum RenderError {
    #[error("Initialization failed: {0}")]
    InitializationFailed(String),
    
    #[error("Resource creation failed: {0}")]
    ResourceCreationFailed(String),
    
    #[error("Invalid operation: {0}")]
    InvalidOperation(String),
    
    #[error("Shader compilation failed: {0}")]
    ShaderCompilationFailed(String),
    
    #[error("Command execution failed: {0}")]
    CommandExecutionFailed(String),
    
    #[error("Context lost")]
    ContextLost,
    
    #[error("Out of memory")]
    OutOfMemory,
    
    #[error("Unsupported feature: {0}")]
    UnsupportedFeature(String),
    
    #[error("Serialization error: {0}")]
    SerializationError(String),
}

pub type RenderResult<T> = Result<T, RenderError>;