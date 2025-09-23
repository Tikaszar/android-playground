//! Public API functions for rendering operations
//!
//! These functions provide a convenient way to access rendering functionality
//! without needing to manage the renderer instance directly.

use once_cell::sync::Lazy;
use playground_core_types::{Handle, CoreResult};
use playground_core_ecs::{Entity, EntityRef};
use crate::{Renderer, RendererConfig, RendererCapabilities, RendererStats};
use crate::commands::RenderCommand;
use crate::types::*;

/// Global renderer instance
static RENDERER_INSTANCE: Lazy<Handle<Renderer>> = Lazy::new(|| Renderer::new());

/// Get the global renderer instance
pub fn get_renderer_instance() -> CoreResult<&'static Handle<Renderer>> {
    Ok(&*RENDERER_INSTANCE)
}

/// Initialize the renderer with the given configuration
pub async fn initialize_renderer(config: RendererConfig) -> CoreResult<()> {
    get_renderer_instance()?.initialize(config).await
}

/// Shutdown the renderer
pub async fn shutdown_renderer() -> CoreResult<()> {
    get_renderer_instance()?.shutdown().await
}

/// Get renderer capabilities
pub async fn get_capabilities() -> CoreResult<RendererCapabilities> {
    Ok(get_renderer_instance()?.capabilities.read().await.clone())
}

/// Get renderer statistics
pub async fn get_stats() -> CoreResult<RendererStats> {
    Ok(get_renderer_instance()?.stats.read().await.clone())
}

/// Switch to a different rendering backend
pub async fn switch_backend(backend: &str) -> CoreResult<()> {
    get_renderer_instance()?.switch_backend(backend).await
}

/// Submit a frame worth of commands
pub async fn submit_frame(commands: Vec<RenderCommand>) -> CoreResult<()> {
    let payload = bincode::serialize(&commands)
        .map_err(|e| playground_core_types::CoreError::SerializationError(e.to_string()))?;

    let renderer = get_renderer_instance()?;
    let response = renderer.vtable.send_command(
        "renderer",
        "submit_frame".to_string(),
        bytes::Bytes::from(payload)
    ).await?;

    if !response.success {
        return Err(playground_core_types::CoreError::Generic(
            response.error.unwrap_or_else(|| "Failed to submit frame".to_string())
        ));
    }

    Ok(())
}

/// Present the current frame
pub async fn present() -> CoreResult<()> {
    get_renderer_instance()?.present().await
}

// Command buffer operations
#[cfg(feature = "commands")]
pub async fn create_command_buffer() -> CoreResult<ResourceId> {
    let renderer = get_renderer_instance()?;
    let id = renderer.generate_resource_id().await;

    renderer.command_buffers.write().await.insert(id, crate::commands::CommandBufferInfo {
        id,
        state: crate::commands::CommandBufferState::Initial,
        level: crate::commands::CommandBufferLevel::Primary,
    });

    Ok(id)
}

#[cfg(feature = "commands")]
pub async fn begin_recording(buffer_id: ResourceId) -> CoreResult<()> {
    let renderer = get_renderer_instance()?;

    if let Some(buffer) = renderer.command_buffers.write().await.get_mut(&buffer_id) {
        buffer.state = crate::commands::CommandBufferState::Recording;
    }

    *renderer.recording_buffer.write().await = Some(buffer_id);
    Ok(())
}

#[cfg(feature = "commands")]
pub async fn end_recording(buffer_id: ResourceId) -> CoreResult<()> {
    let renderer = get_renderer_instance()?;

    if let Some(buffer) = renderer.command_buffers.write().await.get_mut(&buffer_id) {
        buffer.state = crate::commands::CommandBufferState::Executable;
    }

    *renderer.recording_buffer.write().await = None;
    Ok(())
}

#[cfg(feature = "commands")]
pub async fn submit_commands(buffer_id: ResourceId) -> CoreResult<()> {
    let payload = bincode::serialize(&buffer_id)
        .map_err(|e| playground_core_types::CoreError::SerializationError(e.to_string()))?;

    let renderer = get_renderer_instance()?;
    let response = renderer.vtable.send_command(
        "renderer.commands",
        "submit".to_string(),
        bytes::Bytes::from(payload)
    ).await?;

    if !response.success {
        return Err(playground_core_types::CoreError::Generic(
            response.error.unwrap_or_else(|| "Failed to submit commands".to_string())
        ));
    }

    Ok(())
}

// Render target operations
#[cfg(feature = "targets")]
pub async fn create_render_target(info: crate::resources::RenderTargetInfo) -> CoreResult<ResourceId> {
    let payload = bincode::serialize(&info)
        .map_err(|e| playground_core_types::CoreError::SerializationError(e.to_string()))?;

    let renderer = get_renderer_instance()?;
    let response = renderer.vtable.send_command(
        "renderer.targets",
        "create".to_string(),
        bytes::Bytes::from(payload)
    ).await?;

    if !response.success {
        return Err(playground_core_types::CoreError::Generic(
            response.error.unwrap_or_else(|| "Failed to create render target".to_string())
        ));
    }

    let id: ResourceId = bincode::deserialize(&response.payload.unwrap_or_default())
        .map_err(|e| playground_core_types::CoreError::SerializationError(e.to_string()))?;

    renderer.render_targets.write().await.insert(id, info);
    Ok(id)
}

#[cfg(feature = "targets")]
pub async fn destroy_render_target(id: ResourceId) -> CoreResult<()> {
    let renderer = get_renderer_instance()?;
    renderer.render_targets.write().await.remove(&id);

    let payload = bincode::serialize(&id)
        .map_err(|e| playground_core_types::CoreError::SerializationError(e.to_string()))?;

    let response = renderer.vtable.send_command(
        "renderer.targets",
        "destroy".to_string(),
        bytes::Bytes::from(payload)
    ).await?;

    if !response.success {
        return Err(playground_core_types::CoreError::Generic(
            response.error.unwrap_or_else(|| "Failed to destroy render target".to_string())
        ));
    }

    Ok(())
}

#[cfg(feature = "targets")]
pub async fn set_render_target(id: Option<ResourceId>) -> CoreResult<()> {
    let renderer = get_renderer_instance()?;
    *renderer.current_target.write().await = id;

    let payload = bincode::serialize(&id)
        .map_err(|e| playground_core_types::CoreError::SerializationError(e.to_string()))?;

    let response = renderer.vtable.send_command(
        "renderer.targets",
        "set".to_string(),
        bytes::Bytes::from(payload)
    ).await?;

    if !response.success {
        return Err(playground_core_types::CoreError::Generic(
            response.error.unwrap_or_else(|| "Failed to set render target".to_string())
        ));
    }

    Ok(())
}

#[cfg(feature = "targets")]
pub async fn resize_render_target(id: ResourceId, width: UInt, height: UInt) -> CoreResult<()> {
    #[derive(serde::Serialize)]
    struct ResizePayload {
        id: ResourceId,
        width: UInt,
        height: UInt,
    }

    let payload = bincode::serialize(&ResizePayload { id, width, height })
        .map_err(|e| playground_core_types::CoreError::SerializationError(e.to_string()))?;

    let renderer = get_renderer_instance()?;
    let response = renderer.vtable.send_command(
        "renderer.targets",
        "resize".to_string(),
        bytes::Bytes::from(payload)
    ).await?;

    if !response.success {
        return Err(playground_core_types::CoreError::Generic(
            response.error.unwrap_or_else(|| "Failed to resize render target".to_string())
        ));
    }

    if let Some(target) = renderer.render_targets.write().await.get_mut(&id) {
        target.width = width;
        target.height = height;
    }

    Ok(())
}