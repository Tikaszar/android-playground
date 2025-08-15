use thiserror::Error;

#[derive(Error, Debug)]
pub enum RendererError {
    #[error("Renderer initialization failed: {0}")]
    InitializationFailed(String),
    
    #[error("Invalid resource handle")]
    InvalidHandle,
    
    #[error("Resource creation failed: {0}")]
    ResourceCreationFailed(String),
    
    #[error("Shader compilation failed: {0}")]
    ShaderCompilationFailed(String),
    
    #[error("Pipeline creation failed: {0}")]
    PipelineCreationFailed(String),
    
    #[error("Command buffer error: {0}")]
    CommandBufferError(String),
    
    #[error("Device lost")]
    DeviceLost,
    
    #[error("Out of memory")]
    OutOfMemory,
    
    #[error("Feature not supported: {0}")]
    NotSupported(String),
    
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
    
    #[error("WebGL error: {0}")]
    WebGLError(String),
}