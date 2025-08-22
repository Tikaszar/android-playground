use std::fmt;

#[derive(Debug)]
pub enum UiError {
    NotInitialized,
    ElementNotFound(String),
    InvalidOperation(String),
    RenderingFailed(String),
    EventHandlingFailed(String),
    LayoutFailed(String),
    StyleError(String),
    SerializationError(String),
}

impl fmt::Display for UiError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NotInitialized => write!(f, "UI system not initialized"),
            Self::ElementNotFound(id) => write!(f, "Element not found: {}", id),
            Self::InvalidOperation(msg) => write!(f, "Invalid operation: {}", msg),
            Self::RenderingFailed(msg) => write!(f, "Rendering failed: {}", msg),
            Self::EventHandlingFailed(msg) => write!(f, "Event handling failed: {}", msg),
            Self::LayoutFailed(msg) => write!(f, "Layout failed: {}", msg),
            Self::StyleError(msg) => write!(f, "Style error: {}", msg),
            Self::SerializationError(msg) => write!(f, "Serialization error: {}", msg),
        }
    }
}

impl std::error::Error for UiError {}

pub type UiResult<T> = Result<T, UiError>;