//! Input API methods - delegate to VTable, NO LOGIC!

use bytes::Bytes;
use playground_core_types::{CoreResult, CoreError};
use crate::Console;

#[cfg(feature = "input")]
use crate::input::InputEvent;

#[cfg(feature = "input")]
impl Console {
    /// Read a line of input (delegated to systems/console via VTable)
    pub async fn read_line(&self) -> CoreResult<String> {
        let response = self.vtable.send_command(
            "console.input",
            "read_line".to_string(),
            Bytes::new()
        ).await?;
        
        if !response.success {
            return Err(CoreError::Generic(
                response.error.unwrap_or_else(|| "Failed to read line".to_string())
            ));
        }
        
        let payload = response.payload.ok_or(CoreError::UnexpectedResponse)?;
        bincode::deserialize(&payload)
            .map_err(|e| CoreError::DeserializationError(e.to_string()))
    }
    
    /// Read an input event (delegated to systems/console via VTable)
    pub async fn read_event(&self) -> CoreResult<Option<InputEvent>> {
        let response = self.vtable.send_command(
            "console.input",
            "read_event".to_string(),
            Bytes::new()
        ).await?;
        
        if !response.success {
            return Err(CoreError::Generic(
                response.error.unwrap_or_else(|| "Failed to read event".to_string())
            ));
        }
        
        let payload = response.payload.ok_or(CoreError::UnexpectedResponse)?;
        bincode::deserialize(&payload)
            .map_err(|e| CoreError::DeserializationError(e.to_string()))
    }
    
    /// Check if input is available (delegated to systems/console via VTable)
    pub async fn has_input(&self) -> CoreResult<bool> {
        let response = self.vtable.send_command(
            "console.input",
            "has_input".to_string(),
            Bytes::new()
        ).await?;
        
        if !response.success {
            return Err(CoreError::Generic(
                response.error.unwrap_or_else(|| "Failed to check input".to_string())
            ));
        }
        
        let payload = response.payload.ok_or(CoreError::UnexpectedResponse)?;
        bincode::deserialize(&payload)
            .map_err(|e| CoreError::DeserializationError(e.to_string()))
    }
}