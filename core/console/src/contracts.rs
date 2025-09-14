//! Generic console and logging contracts

use async_trait::async_trait;
use crate::types::*;
use std::error::Error;
use std::time::SystemTime;

/// Generic contract for console output operations
#[async_trait]
pub trait ConsoleContract: Send + Sync {
    /// Write text to the console
    async fn write(&self, text: &str) -> Result<(), Box<dyn Error>>;
    
    /// Write text with a specific style hint
    async fn write_styled(&self, text: &str, style: OutputStyle) -> Result<(), Box<dyn Error>>;
    
    /// Write a line (text + newline)
    async fn write_line(&self, text: &str) -> Result<(), Box<dyn Error>>;
    
    /// Clear the console (if supported)
    async fn clear(&self) -> Result<(), Box<dyn Error>>;
    
    /// Update or create a progress indicator
    async fn update_progress(&self, progress: Progress) -> Result<(), Box<dyn Error>>;
    
    /// Remove a progress indicator
    async fn clear_progress(&self, id: &str) -> Result<(), Box<dyn Error>>;
    
    /// Get the capabilities of this console
    async fn capabilities(&self) -> ConsoleCapabilities;
    
    /// Flush any buffered output
    async fn flush(&self) -> Result<(), Box<dyn Error>>;
}

/// Generic contract for logging operations
#[async_trait]
pub trait LoggingContract: Send + Sync {
    /// Log an entry
    async fn log(&self, entry: LogEntry) -> Result<(), Box<dyn Error>>;
    
    /// Log with just level and message (convenience)
    async fn log_simple(&self, level: LogLevel, message: String) -> Result<(), Box<dyn Error>> {
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
    async fn log_component(&self, component: &str, level: LogLevel, message: String) -> Result<(), Box<dyn Error>> {
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
    async fn get_recent_logs(&self, count: usize) -> Result<Vec<LogEntry>, Box<dyn Error>>;
    
    /// Query logs by component (if supported)
    async fn get_component_logs(&self, component: &str, count: usize) -> Result<Vec<LogEntry>, Box<dyn Error>>;
    
    /// Clear all logs (if supported)
    async fn clear_logs(&self) -> Result<(), Box<dyn Error>>;
    
    /// Get minimum log level that will be recorded
    async fn get_log_level(&self) -> LogLevel;
    
    /// Set minimum log level
    async fn set_log_level(&self, level: LogLevel) -> Result<(), Box<dyn Error>>;
}

/// Generic contract for console input (if supported)
#[async_trait]
pub trait InputContract: Send + Sync {
    /// Read a line of input
    async fn read_line(&self) -> Result<String, Box<dyn Error>>;
    
    /// Read a single key/event (non-blocking if possible)
    async fn read_event(&self) -> Result<Option<InputEvent>, Box<dyn Error>>;
    
    /// Check if input is available
    async fn has_input(&self) -> bool;
}

/// Combined console contract for implementations that provide everything
#[async_trait]
pub trait FullConsoleContract: ConsoleContract + LoggingContract + InputContract + Send + Sync {}