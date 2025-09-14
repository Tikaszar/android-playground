//! Command processor pattern for console operations

use crate::types::*;
use playground_core_ecs::{EcsResult, EcsError};
use serde::{Deserialize, Serialize};
use async_trait::async_trait;

/// Commands for console operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConsoleCommand {
    /// Write text to console
    Write { text: String },
    /// Write styled text
    WriteStyled { text: String, style: OutputStyle },
    /// Write a line
    WriteLine { text: String },
    /// Clear the console
    Clear,
    /// Update progress
    UpdateProgress { progress: Progress },
    /// Clear progress
    ClearProgress { id: String },
    /// Get capabilities
    GetCapabilities,
    /// Flush output
    Flush,
    /// Log an entry
    Log { entry: LogEntry },
    /// Get recent logs
    GetRecentLogs { count: usize },
    /// Get component logs
    GetComponentLogs { component: String, count: usize },
    /// Clear logs
    ClearLogs,
    /// Get log level
    GetLogLevel,
    /// Set log level
    SetLogLevel { level: LogLevel },
    /// Read line (for input)
    ReadLine,
    /// Read event
    ReadEvent,
    /// Check for input
    HasInput,
}

/// Responses from console commands
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConsoleResponse {
    /// Generic success
    Success,
    /// Capabilities response
    Capabilities(ConsoleCapabilities),
    /// Log entries response
    Logs(Vec<LogEntry>),
    /// Log level response
    LogLevel(LogLevel),
    /// Input line response
    InputLine(String),
    /// Input event response
    InputEvent(Option<InputEvent>),
    /// Has input response
    HasInput(bool),
    /// Error response
    Error(String),
}

/// Trait for handling console commands
#[async_trait]
pub trait ConsoleCommandHandler: Send + Sync {
    /// Handle a console command
    async fn handle_command(&self, command: ConsoleCommand) -> EcsResult<ConsoleResponse>;
}

/// Static functions for console access through ECS
/// These will send commands through the World command processor
pub mod console_access {
    use super::*;
    use playground_core_types::{Shared, shared};
    use tokio::sync::mpsc;
    use once_cell::sync::Lazy;
    
    type CommandSender = mpsc::Sender<(ConsoleCommand, mpsc::Sender<EcsResult<ConsoleResponse>>)>;
    static COMMAND_SENDER: Lazy<Shared<Option<CommandSender>>> = Lazy::new(|| shared(None));
    
    /// Register the console command processor
    pub async fn register_processor(sender: CommandSender) -> EcsResult<()> {
        let mut guard = COMMAND_SENDER.write().await;
        *guard = Some(sender);
        Ok(())
    }
    
    /// Internal helper to send commands
    async fn send_command(cmd: ConsoleCommand) -> EcsResult<ConsoleResponse> {
        let sender = {
            let guard = COMMAND_SENDER.read().await;
            guard.as_ref().ok_or(EcsError::NotInitialized)?.clone()
        };
        
        let (response_tx, mut response_rx) = mpsc::channel(1);
        sender.send((cmd, response_tx)).await
            .map_err(|_| EcsError::SendError)?;
        
        response_rx.recv().await
            .ok_or(EcsError::ReceiveError)?
    }
    
    /// Write text to console
    pub async fn write(text: &str) -> EcsResult<()> {
        send_command(ConsoleCommand::Write { text: text.to_string() }).await?;
        Ok(())
    }
    
    /// Write styled text
    pub async fn write_styled(text: &str, style: OutputStyle) -> EcsResult<()> {
        send_command(ConsoleCommand::WriteStyled { 
            text: text.to_string(), 
            style 
        }).await?;
        Ok(())
    }
    
    /// Write a line
    pub async fn write_line(text: &str) -> EcsResult<()> {
        send_command(ConsoleCommand::WriteLine { text: text.to_string() }).await?;
        Ok(())
    }
    
    /// Clear console
    pub async fn clear() -> EcsResult<()> {
        send_command(ConsoleCommand::Clear).await?;
        Ok(())
    }
    
    /// Log a message
    pub async fn log(level: LogLevel, message: String) -> EcsResult<()> {
        let entry = LogEntry {
            timestamp: std::time::SystemTime::now(),
            level,
            component: None,
            message,
            data: None,
            correlation_id: None,
        };
        send_command(ConsoleCommand::Log { entry }).await?;
        Ok(())
    }
    
    /// Log with component
    pub async fn log_component(component: &str, level: LogLevel, message: String) -> EcsResult<()> {
        let entry = LogEntry {
            timestamp: std::time::SystemTime::now(),
            level,
            component: Some(component.to_string()),
            message,
            data: None,
            correlation_id: None,
        };
        send_command(ConsoleCommand::Log { entry }).await?;
        Ok(())
    }
    
    /// Get recent logs
    pub async fn get_recent_logs(count: usize) -> EcsResult<Vec<LogEntry>> {
        match send_command(ConsoleCommand::GetRecentLogs { count }).await? {
            ConsoleResponse::Logs(logs) => Ok(logs),
            ConsoleResponse::Error(e) => Err(EcsError::Generic(e)),
            _ => Err(EcsError::Generic("Unexpected response".to_string())),
        }
    }
}