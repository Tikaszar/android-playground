//! Basic types for console operations

use serde::{Deserialize, Serialize};

/// Log level enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum LogLevel {
    Fatal,
    Error,
    Warning,
    Info,
    Debug,
    Trace,
}

impl Default for LogLevel {
    fn default() -> Self {
        LogLevel::Info
    }
}

/// Log entry structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogEntry {
    pub timestamp: u64,  // Milliseconds since epoch
    pub level: LogLevel,
    pub component: Option<String>,
    pub message: String,
    #[cfg(feature = "structured")]
    pub data: Option<serde_json::Value>,
    pub correlation_id: Option<String>,
}

/// Output style for console text
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OutputStyle {
    Plain,
    Emphasis,
    Success,
    Warning,
    Error,
    Dimmed,
    Code,
}

/// Progress indicator
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Progress {
    pub id: String,
    pub label: String,
    pub current: f32,
    pub message: Option<String>,
    pub indeterminate: bool,
}

/// Console capabilities
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsoleCapabilities {
    pub color: bool,
    pub styling: bool,
    pub progress: bool,
    pub clear: bool,
    pub cursor_control: bool,
    pub input: bool,
    pub width: Option<u32>,
    pub height: Option<u32>,
}

impl Default for ConsoleCapabilities {
    fn default() -> Self {
        Self {
            color: false,
            styling: false,
            progress: false,
            clear: false,
            cursor_control: false,
            input: false,
            width: None,
            height: None,
        }
    }
}