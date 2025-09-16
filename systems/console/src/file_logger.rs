//! File-based logging implementation

use playground_core_types::{CoreResult, CoreError};
use playground_core_console::LogEntry;
use tokio::fs::File;
use tokio::io::AsyncWriteExt;
use std::path::PathBuf;

/// File logger for persistent logging
pub struct FileLogger {
    file: File,
    path: PathBuf,
}

impl FileLogger {
    pub async fn new(filename: &str) -> CoreResult<Self> {
        // Ensure logs directory exists
        tokio::fs::create_dir_all("logs").await
            .map_err(|e| CoreError::Io(e.to_string()))?;
        
        let path = PathBuf::from("logs").join(filename);
        let file = File::create(&path).await
            .map_err(|e| CoreError::Io(e.to_string()))?;
        
        Ok(Self { file, path })
    }
    
    pub async fn log(&mut self, entry: &LogEntry) -> CoreResult<()> {
        let timestamp = chrono::DateTime::<chrono::Utc>::from_timestamp_millis(entry.timestamp as i64)
            .map(|dt| dt.format("%Y-%m-%d %H:%M:%S%.3f").to_string())
            .unwrap_or_else(|| "????-??-?? ??:??:??.???".to_string());
        
        let level_str = format!("{:?}", entry.level);
        
        let log_line = if let Some(component) = &entry.component {
            format!("[{}] [{}] [{}] {}\n", timestamp, level_str, component, entry.message)
        } else {
            format!("[{}] [{}] {}\n", timestamp, level_str, entry.message)
        };
        
        self.file.write_all(log_line.as_bytes()).await
            .map_err(|e| CoreError::Io(e.to_string()))?;
        
        self.file.flush().await
            .map_err(|e| CoreError::Io(e.to_string()))?;
        
        Ok(())
    }
    
    pub async fn rotate(&mut self) -> CoreResult<()> {
        // Close current file
        self.file.flush().await
            .map_err(|e| CoreError::Io(e.to_string()))?;
        
        // Create new filename with timestamp
        let timestamp = chrono::Local::now().format("%Y%m%d_%H%M%S");
        let stem = self.path.file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("log");
        let new_filename = format!("{}_{}.log", stem, timestamp);
        
        // Create new file
        let new_path = PathBuf::from("logs").join(&new_filename);
        let new_file = File::create(&new_path).await
            .map_err(|e| CoreError::Io(e.to_string()))?;
        
        self.file = new_file;
        self.path = new_path;
        
        Ok(())
    }
}