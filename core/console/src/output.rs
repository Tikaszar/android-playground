//! Output API methods - delegate to VTable, NO LOGIC!

use bytes::Bytes;
use playground_core_types::{CoreResult, CoreError};
use crate::Console;

#[cfg(feature = "output")]
impl Console {
    /// Write text to console (delegated to systems/console via VTable)
    pub async fn write(&self, text: &str) -> CoreResult<()> {
        let payload = bincode::serialize(&text)
            .map_err(|e| CoreError::SerializationError(e.to_string()))?;
        
        let response = self.vtable.send_command(
            "console.output",
            "write".to_string(),
            Bytes::from(payload)
        ).await?;
        
        if !response.success {
            return Err(CoreError::Generic(
                response.error.unwrap_or_else(|| "Failed to write to console".to_string())
            ));
        }
        
        Ok(())
    }
    
    /// Write styled text (delegated to systems/console via VTable)
    #[cfg(feature = "styling")]
    pub async fn write_styled(&self, text: &str, style: crate::OutputStyle) -> CoreResult<()> {
        let payload = bincode::serialize(&(text, style))
            .map_err(|e| CoreError::SerializationError(e.to_string()))?;
        
        let response = self.vtable.send_command(
            "console.output",
            "write_styled".to_string(),
            Bytes::from(payload)
        ).await?;
        
        if !response.success {
            return Err(CoreError::Generic(
                response.error.unwrap_or_else(|| "Failed to write styled text".to_string())
            ));
        }
        
        Ok(())
    }
    
    /// Write a line (delegated to systems/console via VTable)
    pub async fn write_line(&self, text: &str) -> CoreResult<()> {
        let payload = bincode::serialize(&text)
            .map_err(|e| CoreError::SerializationError(e.to_string()))?;
        
        let response = self.vtable.send_command(
            "console.output",
            "write_line".to_string(),
            Bytes::from(payload)
        ).await?;
        
        if !response.success {
            return Err(CoreError::Generic(
                response.error.unwrap_or_else(|| "Failed to write line".to_string())
            ));
        }
        
        Ok(())
    }
    
    /// Clear console (delegated to systems/console via VTable)
    pub async fn clear(&self) -> CoreResult<()> {
        let response = self.vtable.send_command(
            "console.output",
            "clear".to_string(),
            Bytes::new()
        ).await?;
        
        if !response.success {
            return Err(CoreError::Generic(
                response.error.unwrap_or_else(|| "Failed to clear console".to_string())
            ));
        }
        
        Ok(())
    }
    
    /// Flush output buffer (delegated to systems/console via VTable)
    pub async fn flush(&self) -> CoreResult<()> {
        let response = self.vtable.send_command(
            "console.output",
            "flush".to_string(),
            Bytes::new()
        ).await?;
        
        if !response.success {
            return Err(CoreError::Generic(
                response.error.unwrap_or_else(|| "Failed to flush console".to_string())
            ));
        }
        
        Ok(())
    }
}