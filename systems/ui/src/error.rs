use thiserror::Error;

#[derive(Error, Debug)]
pub enum UiError {
    #[error("UI system already initialized")]
    AlreadyInitialized,
    
    #[error("UI system not initialized")]
    NotInitialized,
    
    #[error("Element not found: {0}")]
    ElementNotFound(String),
    
    #[error("Theme not found: {0}")]
    ThemeNotFound(String),
    
    #[error("Layout error: {0}")]
    LayoutError(String),
    
    #[error("Input error: {0}")]
    InputError(String),
    
    #[error("Rendering error: {0}")]
    RenderingError(String),
    
    #[error("Terminal error: {0}")]
    TerminalError(String),
    
    #[error("ECS error: {0}")]
    EcsError(String),
    
    #[error("Serialization error: {0}")]
    SerializationError(String),
    
    #[error("Network error: {0}")]
    NetworkError(String),
    
    #[error("Creation failed: {0}")]
    CreationFailed(String),
    
    #[error("Invalid operation: {0}")]
    InvalidOperation(String),
}

pub type UiResult<T> = Result<T, UiError>;