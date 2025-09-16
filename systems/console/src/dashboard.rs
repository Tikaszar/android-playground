//! Dashboard implementation for monitoring

use std::collections::HashMap;
use playground_core_types::{CoreResult, CoreError, Shared, shared};
use playground_core_console::{LogEntry, LogLevel};
use tokio::fs::File;
use tokio::io::AsyncWriteExt;

/// Dashboard for monitoring and logging
pub struct Dashboard {
    /// Recent log entries
    log_entries: Shared<Vec<LogEntry>>,
    
    /// Component-specific log files
    component_log_files: Shared<HashMap<String, File>>,
    
    /// Main log file
    main_log_file: Shared<Option<File>>,
}

impl Dashboard {
    pub async fn new() -> CoreResult<Self> {
        // Create logs directory if it doesn't exist
        tokio::fs::create_dir_all("logs").await
            .map_err(|e| CoreError::Io(e.to_string()))?;
        
        Ok(Self {
            log_entries: shared(Vec::new()),
            component_log_files: shared(HashMap::new()),
            main_log_file: shared(None),
        })
    }
    
    pub async fn init_log_file(&self) -> CoreResult<()> {
        let timestamp = chrono::Local::now().format("%Y%m%d_%H%M%S");
        let filename = format!("logs/main_{}.log", timestamp);
        
        let file = File::create(&filename).await
            .map_err(|e| CoreError::Io(e.to_string()))?;
        
        let mut main_file = self.main_log_file.write().await;
        *main_file = Some(file);
        
        Ok(())
    }
    
    pub async fn log(&self, entry: &LogEntry) -> CoreResult<()> {
        // Store in memory
        {
            let mut entries = self.log_entries.write().await;
            entries.push(entry.clone());
            
            // Keep only last 1000 entries in memory
            if entries.len() > 1000 {
                let drain_count = entries.len() - 1000;
                entries.drain(0..drain_count);
            }
        }
        
        // Format log line
        let timestamp = chrono::DateTime::<chrono::Utc>::from_timestamp_millis(entry.timestamp as i64)
            .map(|dt| dt.format("%Y-%m-%d %H:%M:%S%.3f").to_string())
            .unwrap_or_else(|| "????-??-?? ??:??:??.???".to_string());
        
        let level_str = match entry.level {
            LogLevel::Trace => "TRACE",
            LogLevel::Debug => "DEBUG",
            LogLevel::Info => "INFO ",
            LogLevel::Warning => "WARN ",
            LogLevel::Error => "ERROR",
            LogLevel::Fatal => "FATAL",
        };
        
        let log_line = if let Some(component) = &entry.component {
            format!("[{}] [{}] [{}] {}\n", timestamp, level_str, component, entry.message)
        } else {
            format!("[{}] [{}] {}\n", timestamp, level_str, entry.message)
        };
        
        // Write to main log file
        {
            let mut main_file = self.main_log_file.write().await;
            if let Some(file) = main_file.as_mut() {
                let _ = file.write_all(log_line.as_bytes()).await;
                let _ = file.flush().await;
            }
        }
        
        // Write to component-specific log file if component is specified
        if let Some(component) = &entry.component {
            let mut files = self.component_log_files.write().await;
            
            if !files.contains_key(component) {
                // Create component log file
                let timestamp = chrono::Local::now().format("%Y%m%d_%H%M%S");
                let filename = format!("logs/{}_{}.log", 
                    component.replace('/', "_").replace('\\', "_"),
                    timestamp
                );
                
                if let Ok(file) = File::create(&filename).await {
                    files.insert(component.clone(), file);
                }
            }
            
            if let Some(file) = files.get_mut(component) {
                let _ = file.write_all(log_line.as_bytes()).await;
                let _ = file.flush().await;
            }
        }
        
        Ok(())
    }
    
    pub async fn get_recent_logs(&self, count: usize) -> Vec<LogEntry> {
        let entries = self.log_entries.read().await;
        entries.iter()
            .rev()
            .take(count)
            .cloned()
            .collect()
    }
    
    pub async fn render(&self) -> String {
        let entries = self.log_entries.read().await;
        
        let mut output = String::new();
        output.push_str("╔════════════════════════════════════════╗\n");
        output.push_str("║           SYSTEM DASHBOARD             ║\n");
        output.push_str("╠════════════════════════════════════════╣\n");
        
        // Show last 10 logs
        output.push_str("║ Recent Logs:                           ║\n");
        for entry in entries.iter().rev().take(10) {
            let timestamp = chrono::DateTime::<chrono::Utc>::from_timestamp_millis(entry.timestamp as i64)
                .map(|dt| dt.format("%H:%M:%S").to_string())
                .unwrap_or_else(|| "??:??:??".to_string());
            
            let level_emoji = match entry.level {
                LogLevel::Trace => "🔍",
                LogLevel::Debug => "🐛",
                LogLevel::Info => "ℹ️",
                LogLevel::Warning => "⚠️",
                LogLevel::Error => "❌",
                LogLevel::Fatal => "💀",
            };
            
            let msg = if entry.message.len() > 30 {
                format!("{}...", &entry.message[..27])
            } else {
                entry.message.clone()
            };
            
            let line = format!("║ {} {} {:<30} ║", timestamp, level_emoji, msg);
            output.push_str(&line);
            output.push('\n');
        }
        
        output.push_str("╚════════════════════════════════════════╝\n");
        output
    }
}