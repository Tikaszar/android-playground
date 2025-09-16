//! Terminal implementation - actual logic for terminal operations

use std::io::{self, Write};
use std::collections::VecDeque;
use playground_core_types::{CoreResult, CoreError, Shared, shared};
use playground_core_console::{LogEntry, LogLevel, OutputStyle, Progress};
use crossterm::{
    ExecutableCommand,
    cursor,
    terminal::{self, ClearType},
    style::{Color, SetForegroundColor, ResetColor}
};

#[cfg(feature = "input")]
use playground_core_console::InputEvent;

const MAX_LOG_ENTRIES: usize = 1000;

/// Terminal implementation with actual logic
pub struct Terminal {
    enabled: bool,
    log_entries: Shared<VecDeque<LogEntry>>,
    log_level: Shared<LogLevel>,
}

impl Terminal {
    pub fn new(enabled: bool) -> Self {
        Self {
            enabled,
            log_entries: shared(VecDeque::with_capacity(MAX_LOG_ENTRIES)),
            log_level: shared(LogLevel::Info),
        }
    }
    
    pub async fn write(&self, text: &str) -> CoreResult<()> {
        if !self.enabled {
            return Ok(());
        }
        print!("{}", text);
        io::stdout().flush().map_err(CoreError::from)?;
        Ok(())
    }
    
    pub async fn write_styled(&self, text: &str, style: OutputStyle) -> CoreResult<()> {
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
    
    pub async fn write_line(&self, text: &str) -> CoreResult<()> {
        if !self.enabled {
            return Ok(());
        }
        println!("{}", text);
        Ok(())
    }
    
    pub async fn clear(&self) -> CoreResult<()> {
        if !self.enabled {
            return Ok(());
        }
        io::stdout().execute(terminal::Clear(ClearType::All)).map_err(CoreError::from)?;
        io::stdout().execute(cursor::MoveTo(0, 0)).map_err(CoreError::from)?;
        Ok(())
    }
    
    pub async fn flush(&self) -> CoreResult<()> {
        io::stdout().flush().map_err(CoreError::from)?;
        Ok(())
    }
    
    pub async fn log(&self, entry: &LogEntry) -> CoreResult<()> {
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
            let timestamp = chrono::DateTime::<chrono::Utc>::from_timestamp_millis(entry.timestamp as i64)
                .map(|dt| dt.format("%H:%M:%S%.3f").to_string())
                .unwrap_or_else(|| "??:??:??.???".to_string());
            
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
    
    pub async fn get_recent_logs(&self, count: usize) -> Vec<LogEntry> {
        let entries = self.log_entries.read().await;
        entries.iter().rev().take(count).cloned().collect()
    }
    
    pub async fn get_component_logs(&self, component: &str, count: usize) -> Vec<LogEntry> {
        let entries = self.log_entries.read().await;
        entries.iter()
            .rev()
            .filter(|e| e.component.as_deref() == Some(component))
            .take(count)
            .cloned()
            .collect()
    }
    
    pub async fn clear_logs(&self) -> CoreResult<()> {
        let mut entries = self.log_entries.write().await;
        entries.clear();
        Ok(())
    }
    
    pub async fn get_log_level(&self) -> LogLevel {
        *self.log_level.read().await
    }
    
    pub async fn set_log_level(&self, level: LogLevel) -> CoreResult<()> {
        let mut current = self.log_level.write().await;
        *current = level;
        Ok(())
    }
    
    pub async fn update_progress(&self, progress: Progress) -> CoreResult<()> {
        if !self.enabled {
            return Ok(());
        }
        
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
    
    pub async fn clear_progress(&self, _id: &str) -> CoreResult<()> {
        // For terminal, just clear the line
        if self.enabled {
            print!("\r\x1B[K");
            io::stdout().flush().map_err(CoreError::from)?;
        }
        Ok(())
    }
    
    pub async fn clear_all_progress(&self) -> CoreResult<()> {
        if self.enabled {
            print!("\r\x1B[K");
            io::stdout().flush().map_err(CoreError::from)?;
        }
        Ok(())
    }
    
    pub async fn read_line(&self) -> CoreResult<String> {
        let mut line = String::new();
        io::stdin().read_line(&mut line).map_err(CoreError::from)?;
        Ok(line.trim_end().to_string())
    }
    
    #[cfg(feature = "input")]
    pub async fn read_event(&self) -> CoreResult<Option<InputEvent>> {
        // For now, just return None - full event handling would require crossterm event loop
        Ok(None)
    }
    
    #[cfg(not(feature = "input"))]
    pub async fn read_event(&self) -> CoreResult<Option<()>> {
        Ok(None)
    }
    
    pub async fn has_input(&self) -> bool {
        // Would need crossterm event polling
        false
    }
}