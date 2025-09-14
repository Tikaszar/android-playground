//! Generic console and logging contracts

use async_trait::async_trait;
use crate::types::*;
use playground_core_types::{CoreError, CoreResult};
use std::time::SystemTime;

/// Generic contract for console output operations
#[async_trait]
pub trait ConsoleContract: Send + Sync {
    /// Write text to the console
    async fn write(&self, text: &str) -> CoreResult<()>;
    
    /// Write text with a specific style hint
    async fn write_styled(&self, text: &str, style: OutputStyle) -> CoreResult<()>;
    
    /// Write a line (text + newline)
    async fn write_line(&self, text: &str) -> CoreResult<()>;
    
    /// Clear the console (if supported)
    async fn clear(&self) -> CoreResult<()>;
    
    /// Update or create a progress indicator
    async fn update_progress(&self, progress: Progress) -> CoreResult<()>;
    
    /// Remove a progress indicator
    async fn clear_progress(&self, id: &str) -> CoreResult<()>;
    
    /// Get the capabilities of this console
    async fn capabilities(&self) -> ConsoleCapabilities;
    
    /// Flush any buffered output
    async fn flush(&self) -> CoreResult<()>;
}

/// Generic contract for logging operations
#[async_trait]
pub trait LoggingContract: Send + Sync {
    /// Log an entry
    async fn log(&self, entry: LogEntry) -> CoreResult<()>;
    
    /// Log with just level and message (convenience)
    async fn log_simple(&self, level: LogLevel, message: String) -> CoreResult<()> {
        self.log(LogEntry {
            timestamp: SystemTime::now(),
            level,
            component: None,
            message,
            data: None,
            correlation_id: None,
        }).await
    }
    
    /// Log with component context
    async fn log_component(&self, component: &str, level: LogLevel, message: String) -> CoreResult<()> {
        self.log(LogEntry {
            timestamp: SystemTime::now(),
            level,
            component: Some(component.to_string()),
            message,
            data: None,
            correlation_id: None,
        }).await
    }
    
    /// Query recent logs (if supported)
    async fn get_recent_logs(&self, count: usize) -> CoreResult<Vec<LogEntry>>;
    
    /// Query logs by component (if supported)
    async fn get_component_logs(&self, component: &str, count: usize) -> CoreResult<Vec<LogEntry>>;
    
    /// Clear all logs (if supported)
    async fn clear_logs(&self) -> CoreResult<()>;
    
    /// Get minimum log level that will be recorded
    async fn get_log_level(&self) -> LogLevel;
    
    /// Set minimum log level
    async fn set_log_level(&self, level: LogLevel) -> CoreResult<()>;
}

/// Generic contract for console input (if supported)
#[async_trait]
pub trait InputContract: Send + Sync {
    /// Read a line of input
    async fn read_line(&self) -> CoreResult<String>;
    
    /// Read a single key/event (non-blocking if possible)
    async fn read_event(&self) -> CoreResult<Option<InputEvent>>;
    
    /// Check if input is available
    async fn has_input(&self) -> bool;
}

/// Combined console contract for implementations that provide everything
#[async_trait]
pub trait FullConsoleContract: ConsoleContract + LoggingContract + InputContract + Send + Sync {}