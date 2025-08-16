//! UI system error types

use thiserror::Error;

#[derive(Error, Debug)]
pub enum UiError {
    #[error("UI initialization failed: {0}")]
    InitializationFailed(String),
    #[error("Invalid UI component: {0}")]
    InvalidComponent(String),
    #[error("Rendering error: {0}")]
    RenderingError(String),
    #[error("Layout error: {0}")]
    LayoutError(String),
    #[error("Input error: {0}")]
    InputError(String),
    #[error("Theme error: {0}")]
    ThemeError(String),
    #[error("Terminal error: {0}")]
    TerminalError(String),
}

pub type UiResult<T> = Result<T, UiError>;