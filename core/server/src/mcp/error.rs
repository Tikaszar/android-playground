use thiserror::Error;

#[derive(Error, Debug)]
pub enum McpError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("Session not found: {0}")]
    SessionNotFound(String),

    #[error("Tool not found: {0}")]
    ToolNotFound(String),

    #[error("Invalid tool parameters: {0}")]
    InvalidParameters(String),

    #[error("Tool execution failed: {0}")]
    ToolExecutionFailed(String),

    #[error("Client disconnected")]
    ClientDisconnected,

    #[error("Protocol error: {0}")]
    ProtocolError(String),

    #[error("Server error: {0}")]
    ServerError(String),
}

pub type McpResult<T> = Result<T, McpError>;