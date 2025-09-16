//! Client operation delegation methods
//! 
//! All methods delegate to systems/webgl (or other client implementations) via VTable

use bytes::Bytes;
use playground_core_types::{CoreResult, CoreError};
use playground_core_rendering::RenderCommand;
use crate::{Client, ClientConfig, ClientState, ClientStats, RenderTarget};
use crate::input::{InputEvent, KeyCode};

impl Client {
    /// Initialize the client (delegated via VTable)
    pub async fn initialize(&self, config: ClientConfig) -> CoreResult<()> {
        let payload = bincode::serialize(&config)
            .map_err(|e| CoreError::SerializationError(e.to_string()))?;
        
        let response = self.vtable.send_command(
            "client",
            "initialize".to_string(),
            Bytes::from(payload)
        ).await?;
        
        if !response.success {
            return Err(CoreError::Generic(response.error.unwrap_or_else(|| "Failed to initialize client".to_string())));
        }
        
        Ok(())
    }
    
    /// Connect to a server (delegated via VTable)
    pub async fn connect(&self, address: &str) -> CoreResult<()> {
        let payload = address.as_bytes().to_vec();
        
        let response = self.vtable.send_command(
            "client",
            "connect".to_string(),
            Bytes::from(payload)
        ).await?;
        
        if !response.success {
            return Err(CoreError::Generic(response.error.unwrap_or_else(|| "Failed to connect to server".to_string())));
        }
        
        // Update local state
        *self.server_address.write().await = Some(address.to_string());
        *self.state.write().await = ClientState::Connecting;
        
        Ok(())
    }
    
    /// Disconnect from server (delegated via VTable)
    pub async fn disconnect(&self) -> CoreResult<()> {
        let response = self.vtable.send_command(
            "client",
            "disconnect".to_string(),
            Bytes::new()
        ).await?;
        
        if !response.success {
            return Err(CoreError::Generic(response.error.unwrap_or_else(|| "Failed to disconnect from server".to_string())));
        }
        
        // Update local state
        *self.server_address.write().await = None;
        *self.state.write().await = ClientState::Disconnected;
        
        Ok(())
    }
    
    /// Get current client state
    pub async fn state(&self) -> ClientState {
        *self.state.read().await
    }
    
    /// Get client ID
    pub fn id(&self) -> crate::ClientId {
        self.id
    }
    
    /// Send a message to the server (delegated via VTable)
    pub async fn send(&self, data: Vec<u8>) -> CoreResult<()> {
        let response = self.vtable.send_command(
            "client",
            "send".to_string(),
            Bytes::from(data)
        ).await?;
        
        if !response.success {
            return Err(CoreError::Generic(response.error.unwrap_or_else(|| "Failed to send message".to_string())));
        }
        
        // Update stats
        let mut stats = self.stats.write().await;
        stats.messages_sent += 1;
        
        Ok(())
    }
    
    /// Receive a message from the server (delegated via VTable)
    pub async fn receive(&self) -> CoreResult<Option<Vec<u8>>> {
        let response = self.vtable.send_command(
            "client",
            "receive".to_string(),
            Bytes::new()
        ).await?;
        
        if !response.success {
            if response.error.as_ref().map_or(false, |e| e.contains("no message")) {
                return Ok(None);
            }
            return Err(CoreError::Generic(response.error.unwrap_or_else(|| "Failed to receive message".to_string())));
        }
        
        if let Some(payload) = response.payload {
            // Update stats
            let mut stats = self.stats.write().await;
            stats.messages_received += 1;
            stats.bytes_received += payload.len() as u64;
            
            Ok(Some(payload.to_vec()))
        } else {
            Ok(None)
        }
    }
    
    /// Update the client (called each frame) (delegated via VTable)
    pub async fn update(&self, delta_time: f32) -> CoreResult<()> {
        let payload = bincode::serialize(&delta_time)
            .map_err(|e| CoreError::SerializationError(e.to_string()))?;
        
        let response = self.vtable.send_command(
            "client",
            "update".to_string(),
            Bytes::from(payload)
        ).await?;
        
        if !response.success {
            return Err(CoreError::Generic(response.error.unwrap_or_else(|| "Failed to update client".to_string())));
        }
        
        Ok(())
    }
    
    /// Get client statistics
    pub async fn stats(&self) -> ClientStats {
        self.stats.read().await.clone()
    }
    
    /// Get client capabilities
    pub fn capabilities(&self) -> &crate::ClientCapabilities {
        &self.capabilities
    }
}

// Rendering operations (feature-gated)
#[cfg(feature = "rendering")]
impl Client {
    /// Create a render target (delegated via VTable)
    pub async fn create_render_target(&self, target: RenderTarget) -> CoreResult<u32> {
        let payload = bincode::serialize(&target)
            .map_err(|e| CoreError::SerializationError(e.to_string()))?;
        
        let response = self.vtable.send_command(
            "client.render",
            "create_target".to_string(),
            Bytes::from(payload)
        ).await?;
        
        if !response.success {
            return Err(CoreError::Generic(response.error.unwrap_or_else(|| "Failed to create render target".to_string())));
        }
        
        // Deserialize the target ID
        let id: u32 = bincode::deserialize(&response.payload.unwrap_or_default())
            .map_err(|e| CoreError::SerializationError(e.to_string()))?;
        
        // Store in local map
        self.render_targets.write().await.insert(id, target);
        
        Ok(id)
    }
    
    /// Destroy a render target (delegated via VTable)
    pub async fn destroy_render_target(&self, id: u32) -> CoreResult<()> {
        let payload = bincode::serialize(&id)
            .map_err(|e| CoreError::SerializationError(e.to_string()))?;
        
        let response = self.vtable.send_command(
            "client.render",
            "destroy_target".to_string(),
            Bytes::from(payload)
        ).await?;
        
        if !response.success {
            return Err(CoreError::Generic(response.error.unwrap_or_else(|| "Failed to destroy render target".to_string())));
        }
        
        // Remove from local map
        self.render_targets.write().await.remove(&id);
        
        Ok(())
    }
    
    /// Get current render target
    pub async fn current_render_target(&self) -> Option<u32> {
        *self.current_render_target.read().await
    }
    
    /// Set current render target (delegated via VTable)
    pub async fn set_render_target(&self, id: u32) -> CoreResult<()> {
        let payload = bincode::serialize(&id)
            .map_err(|e| CoreError::SerializationError(e.to_string()))?;
        
        let response = self.vtable.send_command(
            "client.render",
            "set_target".to_string(),
            Bytes::from(payload)
        ).await?;
        
        if !response.success {
            return Err(CoreError::Generic(response.error.unwrap_or_else(|| "Failed to set render target".to_string())));
        }
        
        *self.current_render_target.write().await = Some(id);
        
        Ok(())
    }
    
    /// Submit render commands (delegated via VTable)
    pub async fn render(&self, commands: Vec<RenderCommand>) -> CoreResult<()> {
        let payload = bincode::serialize(&commands)
            .map_err(|e| CoreError::SerializationError(e.to_string()))?;
        
        let response = self.vtable.send_command(
            "client.render",
            "submit".to_string(),
            Bytes::from(payload)
        ).await?;
        
        if !response.success {
            return Err(CoreError::Generic(response.error.unwrap_or_else(|| "Failed to submit render commands".to_string())));
        }
        
        // Update stats
        let mut stats = self.stats.write().await;
        stats.total_frames += 1;
        
        Ok(())
    }
    
    /// Present the rendered frame (delegated via VTable)
    pub async fn present(&self) -> CoreResult<()> {
        let response = self.vtable.send_command(
            "client.render",
            "present".to_string(),
            Bytes::new()
        ).await?;
        
        if !response.success {
            return Err(CoreError::Generic(response.error.unwrap_or_else(|| "Failed to present frame".to_string())));
        }
        
        Ok(())
    }
    
    /// Resize render target (delegated via VTable)
    pub async fn resize(&self, id: u32, width: u32, height: u32) -> CoreResult<()> {
        #[derive(serde::Serialize)]
        struct ResizePayload {
            id: u32,
            width: u32,
            height: u32,
        }
        
        let payload = bincode::serialize(&ResizePayload { id, width, height })
            .map_err(|e| CoreError::SerializationError(e.to_string()))?;
        
        let response = self.vtable.send_command(
            "client.render",
            "resize".to_string(),
            Bytes::from(payload)
        ).await?;
        
        if !response.success {
            return Err(CoreError::Generic(response.error.unwrap_or_else(|| "Failed to resize render target".to_string())));
        }
        
        // Update local render target info
        if let Some(target) = self.render_targets.write().await.get_mut(&id) {
            target.width = width;
            target.height = height;
        }
        
        Ok(())
    }
}

// Input operations (feature-gated)
#[cfg(feature = "input")]
impl Client {
    /// Poll for input events (delegated via VTable)
    pub async fn poll_input(&self) -> CoreResult<Vec<InputEvent>> {
        let response = self.vtable.send_command(
            "client.input",
            "poll".to_string(),
            Bytes::new()
        ).await?;
        
        if !response.success {
            return Err(CoreError::Generic(response.error.unwrap_or_else(|| "Failed to poll input".to_string())));
        }
        
        if let Some(payload) = response.payload {
            let events: Vec<InputEvent> = bincode::deserialize(&payload)
                .map_err(|e| CoreError::SerializationError(e.to_string()))?;
            Ok(events)
        } else {
            Ok(Vec::new())
        }
    }
    
    /// Set input capture mode (delegated via VTable)
    pub async fn set_input_capture(&self, capture: bool) -> CoreResult<()> {
        let payload = bincode::serialize(&capture)
            .map_err(|e| CoreError::SerializationError(e.to_string()))?;
        
        let response = self.vtable.send_command(
            "client.input",
            "set_capture".to_string(),
            Bytes::from(payload)
        ).await?;
        
        if !response.success {
            return Err(CoreError::Generic(response.error.unwrap_or_else(|| "Failed to set input capture".to_string())));
        }
        
        Ok(())
    }
    
    /// Check if a key is currently pressed
    pub async fn is_key_pressed(&self, key: KeyCode) -> bool {
        self.input_state.read().await.pressed_keys.contains(&key)
    }
    
    /// Get current pointer position
    pub async fn pointer_position(&self) -> Option<(f32, f32)> {
        self.input_state.read().await.mouse_position
    }
}

// Audio operations (feature-gated)
#[cfg(feature = "audio")]
impl Client {
    /// Play an audio buffer (delegated via VTable)
    pub async fn play_audio(&self, data: Vec<u8>, format: crate::client::AudioFormat) -> CoreResult<u32> {
        #[derive(serde::Serialize)]
        struct PlayAudioPayload {
            format: crate::client::AudioFormat,
            data_len: usize,
        }
        
        let mut payload_bytes = Vec::new();
        let header = PlayAudioPayload {
            format,
            data_len: data.len(),
        };
        
        let header_bytes = bincode::serialize(&header)
            .map_err(|e| CoreError::SerializationError(e.to_string()))?;
        
        payload_bytes.extend_from_slice(&header_bytes);
        payload_bytes.extend_from_slice(&data);
        
        let response = self.vtable.send_command(
            "client.audio",
            "play".to_string(),
            Bytes::from(payload_bytes)
        ).await?;
        
        if !response.success {
            return Err(CoreError::Generic(response.error.unwrap_or_else(|| "Failed to play audio".to_string())));
        }
        
        // Deserialize the track ID
        let id: u32 = bincode::deserialize(&response.payload.unwrap_or_default())
            .map_err(|e| CoreError::SerializationError(e.to_string()))?;
        
        // Store track info
        self.audio_tracks.write().await.insert(id, crate::client::AudioTrackInfo {
            id,
            format,
            duration: None,
            is_playing: true,
        });
        
        Ok(id)
    }
    
    /// Stop audio playback (delegated via VTable)
    pub async fn stop_audio(&self, id: u32) -> CoreResult<()> {
        let payload = bincode::serialize(&id)
            .map_err(|e| CoreError::SerializationError(e.to_string()))?;
        
        let response = self.vtable.send_command(
            "client.audio",
            "stop".to_string(),
            Bytes::from(payload)
        ).await?;
        
        if !response.success {
            return Err(CoreError::Generic(response.error.unwrap_or_else(|| "Failed to stop audio".to_string())));
        }
        
        // Remove track info
        self.audio_tracks.write().await.remove(&id);
        
        Ok(())
    }
    
    /// Set audio volume (delegated via VTable)
    pub async fn set_volume(&self, volume: f32) -> CoreResult<()> {
        let clamped_volume = volume.clamp(0.0, 1.0);
        let payload = bincode::serialize(&clamped_volume)
            .map_err(|e| CoreError::SerializationError(e.to_string()))?;
        
        let response = self.vtable.send_command(
            "client.audio",
            "set_volume".to_string(),
            Bytes::from(payload)
        ).await?;
        
        if !response.success {
            return Err(CoreError::Generic(response.error.unwrap_or_else(|| "Failed to set volume".to_string())));
        }
        
        *self.audio_volume.write().await = clamped_volume;
        
        Ok(())
    }
    
    /// Get current volume
    pub async fn volume(&self) -> f32 {
        *self.audio_volume.read().await
    }
}