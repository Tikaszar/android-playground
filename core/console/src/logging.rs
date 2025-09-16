//! Logging API methods - delegate to VTable, NO LOGIC!

use bytes::Bytes;
use playground_core_types::{CoreResult, CoreError};
use crate::{Console, LogEntry, LogLevel};

#[cfg(feature = "logging")]
impl Console {
    /// Log an entry (delegated to systems/console via VTable)
    pub async fn log(&self, entry: LogEntry) -> CoreResult<()> {
        let payload = bincode::serialize(&entry)
            .map_err(|e| CoreError::SerializationError(e.to_string()))?;
        
        let response = self.vtable.send_command(
            "console.logging",
            "log".to_string(),
            Bytes::from(payload)
        ).await?;
        
        if !response.success {
            return Err(CoreError::Generic(
                response.error.unwrap_or_else(|| "Failed to log entry".to_string())
            ));
        }
        
        Ok(())
    }
    
    /// Simple log with level and message (delegated to systems/console via VTable)
    pub async fn log_simple(&self, level: LogLevel, message: String) -> CoreResult<()> {
        let entry = LogEntry {
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_millis() as u64)
                .unwrap_or(0),
            level,
            component: None,
            message,
            #[cfg(feature = "structured")]
            data: None,
            correlation_id: None,
        };
        
        self.log(entry).await
    }
    
    /// Log with component context (delegated to systems/console via VTable)
    pub async fn log_component(&self, component: &str, level: LogLevel, message: String) -> CoreResult<()> {
        let entry = LogEntry {
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_millis() as u64)
                .unwrap_or(0),
            level,
            component: Some(component.to_string()),
            message,
            #[cfg(feature = "structured")]
            data: None,
            correlation_id: None,
        };
        
        self.log(entry).await
    }
    
    /// Get recent logs (delegated to systems/console via VTable)
    pub async fn get_recent_logs(&self, count: usize) -> CoreResult<Vec<LogEntry>> {
        let payload = bincode::serialize(&count)
            .map_err(|e| CoreError::SerializationError(e.to_string()))?;
        
        let response = self.vtable.send_command(
            "console.logging",
            "get_recent".to_string(),
            Bytes::from(payload)
        ).await?;
        
        if !response.success {
            return Err(CoreError::Generic(
                response.error.unwrap_or_else(|| "Failed to get logs".to_string())
            ));
        }
        
        let payload = response.payload.ok_or(CoreError::UnexpectedResponse)?;
        bincode::deserialize(&payload)
            .map_err(|e| CoreError::DeserializationError(e.to_string()))
    }
    
    /// Get component logs (delegated to systems/console via VTable)
    pub async fn get_component_logs(&self, component: &str, count: usize) -> CoreResult<Vec<LogEntry>> {
        let payload = bincode::serialize(&(component, count))
            .map_err(|e| CoreError::SerializationError(e.to_string()))?;
        
        let response = self.vtable.send_command(
            "console.logging",
            "get_component".to_string(),
            Bytes::from(payload)
        ).await?;
        
        if !response.success {
            return Err(CoreError::Generic(
                response.error.unwrap_or_else(|| "Failed to get component logs".to_string())
            ));
        }
        
        let payload = response.payload.ok_or(CoreError::UnexpectedResponse)?;
        bincode::deserialize(&payload)
            .map_err(|e| CoreError::DeserializationError(e.to_string()))
    }
    
    /// Clear all logs (delegated to systems/console via VTable)
    pub async fn clear_logs(&self) -> CoreResult<()> {
        let response = self.vtable.send_command(
            "console.logging",
            "clear".to_string(),
            Bytes::new()
        ).await?;
        
        if !response.success {
            return Err(CoreError::Generic(
                response.error.unwrap_or_else(|| "Failed to clear logs".to_string())
            ));
        }
        
        Ok(())
    }
    
    /// Get log level (delegated to systems/console via VTable)
    pub async fn get_log_level(&self) -> CoreResult<LogLevel> {
        let response = self.vtable.send_command(
            "console.logging",
            "get_level".to_string(),
            Bytes::new()
        ).await?;
        
        if !response.success {
            return Err(CoreError::Generic(
                response.error.unwrap_or_else(|| "Failed to get log level".to_string())
            ));
        }
        
        let payload = response.payload.ok_or(CoreError::UnexpectedResponse)?;
        bincode::deserialize(&payload)
            .map_err(|e| CoreError::DeserializationError(e.to_string()))
    }
    
    /// Set log level (delegated to systems/console via VTable)
    pub async fn set_log_level(&self, level: LogLevel) -> CoreResult<()> {
        let payload = bincode::serialize(&level)
            .map_err(|e| CoreError::SerializationError(e.to_string()))?;
        
        let response = self.vtable.send_command(
            "console.logging",
            "set_level".to_string(),
            Bytes::from(payload)
        ).await?;
        
        if !response.success {
            return Err(CoreError::Generic(
                response.error.unwrap_or_else(|| "Failed to set log level".to_string())
            ));
        }
        
        Ok(())
    }
}