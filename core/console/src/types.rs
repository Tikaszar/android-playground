//! Generic types for console and logging operations

use serde::{Deserialize, Serialize};
use std::time::SystemTime;

/// Generic log level that any logging system can map to its own levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum LogLevel {
    /// Critical errors that require immediate attention
    Fatal,
    /// Errors that prevent normal operation
    Error,
    /// Warnings about potential issues
    Warning,
    /// Informational messages
    Info,
    /// Detailed debugging information
    Debug,
    /// Extremely verbose tracing information
    Trace,
}

/// Generic log entry that can be rendered by any backend
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogEntry {
    /// When the log was created
    pub timestamp: SystemTime,
    /// Severity level
    pub level: LogLevel,
    /// Optional component/module that generated the log
    pub component: Option<String>,
    /// The log message
    pub message: String,
    /// Optional structured data (JSON-like)
    pub data: Option<serde_json::Value>,
    /// Optional correlation ID for tracing
    pub correlation_id: Option<String>,
}

/// Generic console output style hints (implementations may ignore)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OutputStyle {
    /// Plain text with no formatting
    Plain,
    /// Emphasized text (bold, bright, etc.)
    Emphasis,
    /// Success indication (often green)
    Success,
    /// Warning indication (often yellow)
    Warning,
    /// Error indication (often red)
    Error,
    /// Dimmed/muted text
    Dimmed,
    /// Code or monospace text
    Code,
}

/// Generic progress indication
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Progress {
    /// Unique ID for this progress operation
    pub id: String,
    /// Human-readable label
    pub label: String,
    /// Current progress (0.0 to 1.0)
    pub current: f32,
    /// Optional message about current step
    pub message: Option<String>,
    /// Whether this is an indeterminate progress (spinner)
    pub indeterminate: bool,
}

/// Console capabilities that implementations can report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsoleCapabilities {
    /// Supports colored output
    pub color: bool,
    /// Supports text styling (bold, italic, etc.)
    pub styling: bool,
    /// Supports progress indicators
    pub progress: bool,
    /// Supports clearing the screen
    pub clear: bool,
    /// Supports cursor positioning
    pub cursor_control: bool,
    /// Can receive input
    pub input: bool,
    /// Maximum width in characters (None = unlimited)
    pub width: Option<u32>,
    /// Maximum height in lines (None = unlimited)  
    pub height: Option<u32>,
}

/// Generic input event (if console supports input)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum InputEvent {
    /// Text input
    Text(String),
    /// Key press (generic key code)
    Key(KeyCode),
    /// Mouse/pointer event (if supported)
    Pointer(PointerEvent),
    /// Resize event
    Resize { width: u32, height: u32 },
}

/// Generic key codes (not tied to any specific input system)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum KeyCode {
    Enter,
    Escape,
    Backspace,
    Tab,
    Up,
    Down,
    Left,
    Right,
    Home,
    End,
    PageUp,
    PageDown,
    Delete,
    Function(u8), // F1-F12 as 1-12
    Char(char),
    Control(char), // Ctrl+key combinations
    Alt(char),     // Alt+key combinations
}

/// Generic pointer/mouse events
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PointerEvent {
    pub x: f32,
    pub y: f32,
    pub button: Option<PointerButton>,
    pub event_type: PointerEventType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PointerButton {
    Primary,   // Usually left
    Secondary, // Usually right
    Middle,
    Other(u8),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PointerEventType {
    Move,
    Down,
    Up,
    Click,
    DoubleClick,
    Scroll { delta_x: f32, delta_y: f32 },
}