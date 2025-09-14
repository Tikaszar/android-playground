//! Terminal-specific implementation of console contracts using ANSI/crossterm

use async_trait::async_trait;
use playground_core_types::{CoreError, CoreResult};
use std::io::{self, Write};
use playground_core_console::{
    ConsoleContract, LoggingContract, InputContract,
    LogEntry, LogLevel, OutputStyle, Progress, ConsoleCapabilities, InputEvent
};
use playground_core_types::{Shared, shared};
use std::collections::{HashMap, VecDeque};
use crossterm::{
    ExecutableCommand,
    cursor,
    terminal::{self, ClearType},
    style::{Color, SetForegroundColor, ResetColor}
};

const MAX_LOG_ENTRIES: usize = 1000;

/// Terminal implementation of console contracts using ANSI escape codes
pub struct TerminalConsole {
    /// Whether terminal output is enabled
    enabled: bool,
    /// Recent log entries kept in memory
    log_entries: Shared<VecDeque<LogEntry>>,
    /// Current minimum log level
    log_level: Shared<LogLevel>,
    /// Active progress indicators
    progress_indicators: Shared<HashMap<String, Progress>>,
}

impl TerminalConsole {
    pub fn new(enabled: bool) -> Self {
        Self {
            enabled,
            log_entries: shared(VecDeque::with_capacity(MAX_LOG_ENTRIES)),
            log_level: shared(LogLevel::Info),
            progress_indicators: shared(HashMap::new()),
        }
    }
}

#[async_trait]
impl ConsoleContract for TerminalConsole {
    async fn write(&self, text: &str) -> CoreResult<()> {
        if !self.enabled {
            return Ok(());
        }
        print!("{}", text);
        io::stdout().flush().map_err(CoreError::from)?;
        Ok(())
    }
    
    async fn write_styled(&self, text: &str, style: OutputStyle) -> CoreResult<()> {
        if !self.enabled {
            return Ok(());
        }
        
        let color = match style {
            OutputStyle::Plain => None,
            OutputStyle::Emphasis => Some(Color::White),
            OutputStyle::Success => Some(Color::Green),
            OutputStyle::Warning => Some(Color::Yellow),
            OutputStyle::Error => Some(Color::Red),
            OutputStyle::Dimmed => Some(Color::DarkGrey),
            OutputStyle::Code => Some(Color::Cyan),
        };
        
        if let Some(color) = color {
            io::stdout().execute(SetForegroundColor(color)).map_err(CoreError::from)?;
        }
        
        print!("{}", text);
        
        if color.is_some() {
            io::stdout().execute(ResetColor).map_err(CoreError::from)?;
        }
        
        io::stdout().flush().map_err(CoreError::from)?;
        Ok(())
    }
    
    async fn write_line(&self, text: &str) -> CoreResult<()> {
        if !self.enabled {
            return Ok(());
        }
        println!("{}", text);
        Ok(())
    }
    
    async fn clear(&self) -> CoreResult<()> {
        if !self.enabled {
            return Ok(());
        }
        io::stdout().execute(terminal::Clear(ClearType::All)).map_err(CoreError::from)?;
        io::stdout().execute(cursor::MoveTo(0, 0)).map_err(CoreError::from)?;
        Ok(())
    }
    
    async fn update_progress(&self, progress: Progress) -> CoreResult<()> {
        if !self.enabled {
            return Ok(());
        }
        
        let mut indicators = self.progress_indicators.write().await;
        indicators.insert(progress.id.clone(), progress.clone());
        
        // For terminal, we'll just print inline progress
        if progress.indeterminate {
            print!("\r{}: [spinner] - {:?}", progress.label, progress.message);
        } else {
            let percentage = (progress.current * 100.0) as u32;
            print!("\r{}: [{}%] - {:?}", 
                progress.label, percentage, progress.message);
        }
        io::stdout().flush().map_err(CoreError::from)?;
        
        Ok(())
    }
    
    async fn clear_progress(&self, id: &str) -> CoreResult<()> {
        let mut indicators = self.progress_indicators.write().await;
        indicators.remove(id);
        Ok(())
    }
    
    async fn capabilities(&self) -> ConsoleCapabilities {
        ConsoleCapabilities {
            color: true,
            styling: true,
            progress: true,
            clear: true,
            cursor_control: true,
            input: true,
            width: terminal::size().map(|(w, _)| w as u32).ok(),
            height: terminal::size().map(|(_, h)| h as u32).ok(),
        }
    }
    
    async fn flush(&self) -> CoreResult<()> {
        io::stdout().flush().map_err(CoreError::from)?;
        Ok(())
    }
}

#[async_trait]
impl LoggingContract for TerminalConsole {
    async fn log(&self, entry: LogEntry) -> CoreResult<()> {
        // Store in memory
        {
            let mut entries = self.log_entries.write().await;
            if entries.len() >= MAX_LOG_ENTRIES {
                entries.pop_front();
            }
            entries.push_back(entry.clone());
        }
        
        // Check log level
        let min_level = self.log_level.read().await;
        if entry.level < *min_level {
            return Ok(());
        }
        
        // Print to terminal if enabled
        if self.enabled {
            let timestamp = chrono::DateTime::<chrono::Local>::from(entry.timestamp)
                .format("%H:%M:%S%.3f");
            
            let level_emoji = match entry.level {
                LogLevel::Trace => "🔍",
                LogLevel::Debug => "🐛",
                LogLevel::Info => "ℹ️",
                LogLevel::Warning => "⚠️",
                LogLevel::Error => "❌",
                LogLevel::Fatal => "💀",
            };
            
            if let Some(component) = &entry.component {
                println!("[{}] [{}] {} {}", timestamp, component, level_emoji, entry.message);
            } else {
                println!("[{}] {} {}", timestamp, level_emoji, entry.message);
            }
        }
        
        Ok(())
    }
    
    async fn get_recent_logs(&self, count: usize) -> CoreResult<Vec<LogEntry>> {
        let entries = self.log_entries.read().await;
        Ok(entries.iter().rev().take(count).cloned().collect())
    }
    
    async fn get_component_logs(&self, component: &str, count: usize) -> CoreResult<Vec<LogEntry>> {
        let entries = self.log_entries.read().await;
        Ok(entries.iter()
            .rev()
            .filter(|e| e.component.as_deref() == Some(component))
            .take(count)
            .cloned()
            .collect())
    }
    
    async fn clear_logs(&self) -> CoreResult<()> {
        let mut entries = self.log_entries.write().await;
        entries.clear();
        Ok(())
    }
    
    async fn get_log_level(&self) -> LogLevel {
        *self.log_level.read().await
    }
    
    async fn set_log_level(&self, level: LogLevel) -> CoreResult<()> {
        let mut current = self.log_level.write().await;
        *current = level;
        Ok(())
    }
}

#[async_trait]
impl InputContract for TerminalConsole {
    async fn read_line(&self) -> CoreResult<String> {
        let mut line = String::new();
        io::stdin().read_line(&mut line).map_err(CoreError::from)?;
        Ok(line.trim_end().to_string())
    }
    
    async fn read_event(&self) -> CoreResult<Option<InputEvent>> {
        // For now, just return None - full event handling would require crossterm event loop
        Ok(None)
    }
    
    async fn has_input(&self) -> bool {
        // Would need crossterm event polling
        false
    }
}