//! Renderer operation delegation methods
//!
//! All methods delegate to systems/webgl (or other renderer implementations) via VTable

use bytes::Bytes;
use playground_core_types::{CoreResult, CoreError};
use playground_core_ecs::EntityId;
use crate::{Renderer, RendererConfig};
use crate::types::*;

impl Renderer {
    /// Initialize the renderer (delegated via VTable)
    pub async fn initialize(&self, config: RendererConfig) -> CoreResult<()> {
        let payload = bincode::serialize(&config)
            .map_err(|e| CoreError::SerializationError(e.to_string()))?;

        let response = self.vtable.send_command(
            "renderer",
            "initialize".to_string(),
            Bytes::from(payload)
        ).await?;

        if !response.success {
            return Err(CoreError::Generic(
                response.error.unwrap_or_else(|| "Failed to initialize renderer".to_string())
            ));
        }

        *self.is_initialized.write().await = true;
        Ok(())
    }

    /// Shutdown the renderer (delegated via VTable)
    pub async fn shutdown(&self) -> CoreResult<()> {
        let response = self.vtable.send_command(
            "renderer",
            "shutdown".to_string(),
            Bytes::new()
        ).await?;

        if !response.success {
            return Err(CoreError::Generic(
                response.error.unwrap_or_else(|| "Failed to shutdown renderer".to_string())
            ));
        }

        *self.is_initialized.write().await = false;
        Ok(())
    }

    /// Switch rendering backend (delegated via VTable)
    pub async fn switch_backend(&self, backend: &str) -> CoreResult<()> {
        let payload = backend.as_bytes().to_vec();

        let response = self.vtable.send_command(
            "renderer",
            "switch_backend".to_string(),
            Bytes::from(payload)
        ).await?;

        if !response.success {
            return Err(CoreError::Generic(
                response.error.unwrap_or_else(|| format!("Failed to switch to backend: {}", backend))
            ));
        }

        *self.active_backend.write().await = backend.to_string();
        Ok(())
    }

    /// Begin a new frame (delegated via VTable)
    pub async fn begin_frame(&self, camera: EntityId) -> CoreResult<()> {
        let payload = bincode::serialize(&camera)
            .map_err(|e| CoreError::SerializationError(e.to_string()))?;

        let response = self.vtable.send_command(
            "renderer",
            "begin_frame".to_string(),
            Bytes::from(payload)
        ).await?;

        if !response.success {
            return Err(CoreError::Generic(
                response.error.unwrap_or_else(|| "Failed to begin frame".to_string())
            ));
        }

        Ok(())
    }

    /// End the current frame (delegated via VTable)
    pub async fn end_frame(&self) -> CoreResult<()> {
        let response = self.vtable.send_command(
            "renderer",
            "end_frame".to_string(),
            Bytes::new()
        ).await?;

        if !response.success {
            return Err(CoreError::Generic(
                response.error.unwrap_or_else(|| "Failed to end frame".to_string())
            ));
        }

        // Update stats
        let mut stats = self.stats.write().await;
        stats.frames_rendered += 1;

        Ok(())
    }

    /// Present the rendered frame (delegated via VTable)
    pub async fn present(&self) -> CoreResult<()> {
        let response = self.vtable.send_command(
            "renderer",
            "present".to_string(),
            Bytes::new()
        ).await?;

        if !response.success {
            return Err(CoreError::Generic(
                response.error.unwrap_or_else(|| "Failed to present frame".to_string())
            ));
        }

        Ok(())
    }
}