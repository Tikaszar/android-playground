//! Progress indicator API methods - delegate to VTable, NO LOGIC!

use bytes::Bytes;
use playground_core_types::{CoreResult, CoreError};
use crate::{Console, Progress};

#[cfg(feature = "progress")]
impl Console {
    /// Update or create a progress indicator (delegated to systems/console via VTable)
    pub async fn update_progress(&self, progress: Progress) -> CoreResult<()> {
        let payload = bincode::serialize(&progress)
            .map_err(|e| CoreError::SerializationError(e.to_string()))?;
        
        let response = self.vtable.send_command(
            "console.progress",
            "update".to_string(),
            Bytes::from(payload)
        ).await?;
        
        if !response.success {
            return Err(CoreError::Generic(
                response.error.unwrap_or_else(|| "Failed to update progress".to_string())
            ));
        }
        
        Ok(())
    }
    
    /// Clear a progress indicator (delegated to systems/console via VTable)
    pub async fn clear_progress(&self, id: &str) -> CoreResult<()> {
        let payload = bincode::serialize(&id)
            .map_err(|e| CoreError::SerializationError(e.to_string()))?;
        
        let response = self.vtable.send_command(
            "console.progress",
            "clear".to_string(),
            Bytes::from(payload)
        ).await?;
        
        if !response.success {
            return Err(CoreError::Generic(
                response.error.unwrap_or_else(|| "Failed to clear progress".to_string())
            ));
        }
        
        Ok(())
    }
    
    /// Clear all progress indicators (delegated to systems/console via VTable)
    pub async fn clear_all_progress(&self) -> CoreResult<()> {
        let response = self.vtable.send_command(
            "console.progress",
            "clear_all".to_string(),
            Bytes::new()
        ).await?;
        
        if !response.success {
            return Err(CoreError::Generic(
                response.error.unwrap_or_else(|| "Failed to clear all progress".to_string())
            ));
        }
        
        Ok(())
    }
}