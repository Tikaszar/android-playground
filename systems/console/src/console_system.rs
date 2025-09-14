//! Console system that implements command processor for ECS integration

use async_trait::async_trait;
use std::sync::Arc;
use playground_core_console::{
    ConsoleCommand, ConsoleCommandHandler, ConsoleResponse,
};
use playground_core_ecs::EcsError;
use playground_core_types::CoreResult;
use crate::{TerminalConsole, Dashboard};

/// Console system that handles console commands through ECS
pub struct ConsoleSystem {
    terminal: Arc<TerminalConsole>,
    dashboard: Option<Arc<Dashboard>>,
}

impl ConsoleSystem {
    pub fn new(enable_terminal: bool, enable_dashboard: bool) -> Self {
        let terminal = Arc::new(TerminalConsole::new(enable_terminal));
        
        let dashboard = if enable_dashboard {
            match tokio::runtime::Handle::try_current() {
                Ok(_) => {
                    let dash = tokio::task::block_in_place(|| {
                        tokio::runtime::Handle::current().block_on(async {
                            Dashboard::new(true).await.ok()
                        })
                    });
                    dash.map(Arc::new)
                },
                Err(_) => None,
            }
        } else {
            None
        };
        
        Self {
            terminal,
            dashboard,
        }
    }
    
    pub fn terminal(&self) -> Arc<TerminalConsole> {
        self.terminal.clone()
    }
    
    pub fn dashboard(&self) -> Option<Arc<Dashboard>> {
        self.dashboard.clone()
    }
    
    /// Start the dashboard render loop if enabled
    pub async fn start_dashboard(&self) {
        if let Some(dashboard) = &self.dashboard {
            dashboard.clone().start_render_loop().await;
        }
    }
    
    /// Initialize log files
    pub async fn init_log_files(&self) -> CoreResult<()> {
        if let Some(dashboard) = &self.dashboard {
            dashboard.init_log_file().await?;
        }
        Ok(())
    }
}

#[async_trait]
impl ConsoleCommandHandler for ConsoleSystem {
    async fn handle_command(&self, command: ConsoleCommand) -> Result<ConsoleResponse, EcsError> {
        use ConsoleCommand::*;
        use playground_core_console::{ConsoleContract, LoggingContract, InputContract};
        
        match command {
            // Console output commands
            Write { text } => {
                self.terminal.write(&text).await.map_err(|e| EcsError::Generic(e.to_string()))?;
                Ok(ConsoleResponse::Success)
            },
            WriteStyled { text, style } => {
                self.terminal.write_styled(&text, style).await.map_err(|e| EcsError::Generic(e.to_string()))?;
                Ok(ConsoleResponse::Success)
            },
            WriteLine { text } => {
                self.terminal.write_line(&text).await.map_err(|e| EcsError::Generic(e.to_string()))?;
                Ok(ConsoleResponse::Success)
            },
            Clear => {
                self.terminal.clear().await.map_err(|e| EcsError::Generic(e.to_string()))?;
                Ok(ConsoleResponse::Success)
            },
            UpdateProgress { progress } => {
                self.terminal.update_progress(progress).await.map_err(|e| EcsError::Generic(e.to_string()))?;
                Ok(ConsoleResponse::Success)
            },
            ClearProgress { id } => {
                self.terminal.clear_progress(&id).await.map_err(|e| EcsError::Generic(e.to_string()))?;
                Ok(ConsoleResponse::Success)
            },
            GetCapabilities => {
                let caps = self.terminal.capabilities().await;
                Ok(ConsoleResponse::Capabilities(caps))
            },
            Flush => {
                self.terminal.flush().await.map_err(|e| EcsError::Generic(e.to_string()))?;
                Ok(ConsoleResponse::Success)
            },
            
            // Logging commands
            Log { entry } => {
                self.terminal.log(entry.clone()).await.map_err(|e| EcsError::Generic(e.to_string()))?;
                
                // Also log to dashboard if available
                if let Some(dashboard) = &self.dashboard {
                    dashboard.add_log_entry(entry).await;
                }
                
                Ok(ConsoleResponse::Success)
            },
            GetRecentLogs { count } => {
                let logs = self.terminal.get_recent_logs(count).await.map_err(|e| EcsError::Generic(e.to_string()))?;
                Ok(ConsoleResponse::Logs(logs))
            },
            GetComponentLogs { component, count } => {
                let logs = self.terminal.get_component_logs(&component, count).await.map_err(|e| EcsError::Generic(e.to_string()))?;
                Ok(ConsoleResponse::Logs(logs))
            },
            ClearLogs => {
                self.terminal.clear_logs().await.map_err(|e| EcsError::Generic(e.to_string()))?;
                Ok(ConsoleResponse::Success)
            },
            GetLogLevel => {
                let level = self.terminal.get_log_level().await;
                Ok(ConsoleResponse::LogLevel(level))
            },
            SetLogLevel { level } => {
                self.terminal.set_log_level(level).await.map_err(|e| EcsError::Generic(e.to_string()))?;
                Ok(ConsoleResponse::Success)
            },
            
            // Input commands
            ReadLine => {
                let line = self.terminal.read_line().await.map_err(|e| EcsError::Generic(e.to_string()))?;
                Ok(ConsoleResponse::InputLine(line))
            },
            ReadEvent => {
                let event = self.terminal.read_event().await.map_err(|e| EcsError::Generic(e.to_string()))?;
                Ok(ConsoleResponse::InputEvent(event))
            },
            HasInput => {
                let has_input = self.terminal.has_input().await;
                Ok(ConsoleResponse::HasInput(has_input))
            },
        }
    }
}