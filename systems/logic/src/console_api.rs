//! Public API for console/logging operations
//!
//! This module provides clean functions for console operations that internally
//! use the command processor pattern to communicate with the ConsoleSystem.

use playground_core_console::{
    ConsoleCommand, ConsoleResponse, LogLevel, LogEntry,
    OutputStyle, Progress,
};
use playground_core_types::{CoreResult, CoreError};
use tokio::sync::mpsc;
use tokio::sync::RwLock;

/// Global channel for sending commands to the ConsoleSystem
static CONSOLE_COMMAND_SENDER: RwLock<Option<mpsc::Sender<ConsoleCommand>>> = RwLock::const_new(None);

/// Register the console command channel
pub async fn register_console_channel(sender: mpsc::Sender<ConsoleCommand>) -> CoreResult<()> {
    let mut guard = CONSOLE_COMMAND_SENDER.write().await;
    *guard = Some(sender);
    Ok(())
}

/// Log a message
pub async fn log(level: LogLevel, message: String) -> CoreResult<()> {
    send_command(ConsoleCommand::Log {
        level,
        message,
        component: None,
    }).await?;
    Ok(())
}

/// Log a message with component context
pub async fn log_component(component: &str, level: LogLevel, message: String) -> CoreResult<()> {
    send_command(ConsoleCommand::Log {
        level,
        message,
        component: Some(component.to_string()),
    }).await?;
    Ok(())
}

/// Clear the console
pub async fn clear_console() -> CoreResult<()> {
    send_command(ConsoleCommand::Clear).await?;
    Ok(())
}

/// Set the output style
pub async fn set_output_style(style: OutputStyle) -> CoreResult<()> {
    send_command(ConsoleCommand::SetOutputStyle(style)).await?;
    Ok(())
}

/// Update progress
pub async fn update_progress(progress: Progress) -> CoreResult<()> {
    send_command(ConsoleCommand::UpdateProgress(progress)).await?;
    Ok(())
}

/// Get recent log entries
pub async fn get_recent_logs(count: usize) -> CoreResult<Vec<LogEntry>> {
    match send_command(ConsoleCommand::GetLogs { count }).await? {
        ConsoleResponse::Logs(logs) => Ok(logs),
        _ => Err(CoreError::Generic("Unexpected response from console".to_string())),
    }
}

/// Flush any buffered output
pub async fn flush() -> CoreResult<()> {
    send_command(ConsoleCommand::Flush).await?;
    Ok(())
}

/// Internal function to send commands through the channel
async fn send_command(command: ConsoleCommand) -> CoreResult<ConsoleResponse> {
    let guard = CONSOLE_COMMAND_SENDER.read().await;
    let sender = guard.as_ref()
        .ok_or_else(|| CoreError::NotInitialized("Console command channel not registered".to_string()))?;
    
    // In a real implementation, we'd need a response channel mechanism
    // For now, we'll just send the command and return a placeholder response
    sender.send(command).await
        .map_err(|e| CoreError::Generic(e.to_string()))?;
    
    // This is a placeholder - in reality we'd wait for the response
    Ok(ConsoleResponse::Success)
}

/// Convenience functions for different log levels
pub async fn trace(message: String) -> CoreResult<()> {
    log(LogLevel::Trace, message).await
}

pub async fn debug(message: String) -> CoreResult<()> {
    log(LogLevel::Debug, message).await
}

pub async fn info(message: String) -> CoreResult<()> {
    log(LogLevel::Info, message).await
}

pub async fn warn(message: String) -> CoreResult<()> {
    log(LogLevel::Warn, message).await
}

pub async fn error(message: String) -> CoreResult<()> {
    log(LogLevel::Error, message).await
}