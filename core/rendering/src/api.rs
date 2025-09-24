//! Public API functions for rendering operations using ECS
//!
//! These functions work with entities and components in the ECS,
//! similar to how core/server and core/client work.

use playground_core_types::CoreResult;
use playground_core_ecs::{Entity, EntityRef, get_world};
use crate::types::*;
use crate::components::*;
use crate::commands::RenderCommand;

/// Create a renderer entity with the given configuration
/// Returns the renderer entity
pub async fn create_renderer(config: RendererConfig) -> CoreResult<Entity> {
    let world = get_world().await?;
    let renderer_entity = world.spawn_entity().await?;

    // Add MANDATORY components - always added
    renderer_entity.add_component(RendererConfigComponent(config)).await?;
    renderer_entity.add_component(RendererStatsComponent::default()).await?;
    renderer_entity.add_component(RendererCapabilitiesComponent::default()).await?;
    renderer_entity.add_component(RendererBackend::default()).await?;

    // Add OPTIONAL components based on features
    #[cfg(feature = "targets")]
    renderer_entity.add_component(RenderTargetStorage::default()).await?;

    #[cfg(feature = "shaders")]
    renderer_entity.add_component(ShaderStorage::default()).await?;

    #[cfg(feature = "textures")]
    renderer_entity.add_component(TextureStorage::default()).await?;

    #[cfg(feature = "buffers")]
    renderer_entity.add_component(BufferStorage::default()).await?;

    #[cfg(feature = "uniforms")]
    renderer_entity.add_component(UniformBufferStorage::default()).await?;

    #[cfg(feature = "samplers")]
    renderer_entity.add_component(SamplerStorage::default()).await?;

    #[cfg(feature = "pipelines")]
    renderer_entity.add_component(PipelineStorage::default()).await?;

    #[cfg(feature = "commands")]
    renderer_entity.add_component(CommandBufferStorage::default()).await?;

    #[cfg(feature = "passes")]
    renderer_entity.add_component(RenderPassStorage::default()).await?;

    Ok(renderer_entity)
}

/// Initialize a renderer entity
pub async fn initialize_renderer(_renderer: EntityRef, _config: RendererConfig) -> CoreResult<()> {
    // Stub - systems/webgl does actual initialization
    Ok(())
}

/// Shutdown a renderer entity
pub async fn shutdown_renderer(_renderer: EntityRef) -> CoreResult<()> {
    // Stub - systems/webgl does actual shutdown
    Ok(())
}

/// Get renderer capabilities
pub async fn get_capabilities(_renderer: EntityRef) -> CoreResult<RendererCapabilities> {
    // Stub - systems/webgl maintains actual capabilities
    Ok(RendererCapabilities::default())
}

/// Get renderer statistics
pub async fn get_stats(_renderer: EntityRef) -> CoreResult<RendererStats> {
    // Stub - systems/webgl maintains actual statistics
    Ok(RendererStats::default())
}

/// Switch to a different rendering backend
pub async fn switch_backend(_renderer: EntityRef, _backend: &str) -> CoreResult<()> {
    // Stub - systems/webgl or systems/vulkan handle backend switching
    Ok(())
}

/// Submit a frame worth of commands
pub async fn submit_frame(_renderer: EntityRef, _commands: Vec<RenderCommand>) -> CoreResult<()> {
    // Stub - systems/webgl does actual rendering
    Ok(())
}

/// Present the current frame
pub async fn present(_renderer: EntityRef) -> CoreResult<()> {
    // Stub - systems/webgl does actual presentation
    Ok(())
}

// Command buffer operations
#[cfg(feature = "commands")]
pub async fn create_command_buffer(_renderer: EntityRef) -> CoreResult<ResourceId> {
    // Stub - systems/webgl creates actual command buffer
    Ok(0)
}

#[cfg(feature = "commands")]
pub async fn begin_recording(_renderer: EntityRef, _buffer_id: ResourceId) -> CoreResult<()> {
    // Stub - systems/webgl handles recording
    Ok(())
}

#[cfg(feature = "commands")]
pub async fn end_recording(_renderer: EntityRef, _buffer_id: ResourceId) -> CoreResult<()> {
    // Stub - systems/webgl handles recording
    Ok(())
}

#[cfg(feature = "commands")]
pub async fn submit_commands(_renderer: EntityRef, _buffer_id: ResourceId) -> CoreResult<()> {
    // Stub - systems/webgl executes commands
    Ok(())
}

// Render target operations
#[cfg(feature = "targets")]
pub async fn create_render_target(_renderer: EntityRef, _info: crate::resources::RenderTargetInfo) -> CoreResult<ResourceId> {
    // Stub - systems/webgl creates actual render target
    Ok(0)
}

#[cfg(feature = "targets")]
pub async fn destroy_render_target(_renderer: EntityRef, _id: ResourceId) -> CoreResult<()> {
    // Stub - systems/webgl destroys render target
    Ok(())
}

#[cfg(feature = "targets")]
pub async fn set_render_target(_renderer: EntityRef, _id: Option<ResourceId>) -> CoreResult<()> {
    // Stub - systems/webgl sets active render target
    Ok(())
}

#[cfg(feature = "targets")]
pub async fn resize_render_target(_renderer: EntityRef, _id: ResourceId, _width: UInt, _height: UInt) -> CoreResult<()> {
    // Stub - systems/webgl resizes render target
    Ok(())
}

// Shader operations
#[cfg(feature = "shaders")]
pub async fn compile_shader(_renderer: EntityRef, _source: String, _stage: crate::resources::ShaderStage) -> CoreResult<ResourceId> {
    // Stub - systems/webgl compiles actual shader
    Ok(0)
}

#[cfg(feature = "shaders")]
pub async fn destroy_shader(_renderer: EntityRef, _id: ResourceId) -> CoreResult<()> {
    // Stub - systems/webgl destroys shader
    Ok(())
}

// Texture operations
#[cfg(feature = "textures")]
pub async fn create_texture(_renderer: EntityRef, _info: crate::resources::TextureInfo) -> CoreResult<ResourceId> {
    // Stub - systems/webgl creates actual texture
    Ok(0)
}

#[cfg(feature = "textures")]
pub async fn destroy_texture(_renderer: EntityRef, _id: ResourceId) -> CoreResult<()> {
    // Stub - systems/webgl destroys texture
    Ok(())
}

#[cfg(feature = "textures")]
pub async fn update_texture(_renderer: EntityRef, _id: ResourceId, _data: Vec<u8>) -> CoreResult<()> {
    // Stub - systems/webgl updates texture data
    Ok(())
}

// Buffer operations
#[cfg(feature = "buffers")]
pub async fn create_buffer(_renderer: EntityRef, _info: crate::resources::BufferInfo) -> CoreResult<ResourceId> {
    // Stub - systems/webgl creates actual buffer
    Ok(0)
}

#[cfg(feature = "buffers")]
pub async fn destroy_buffer(_renderer: EntityRef, _id: ResourceId) -> CoreResult<()> {
    // Stub - systems/webgl destroys buffer
    Ok(())
}

#[cfg(feature = "buffers")]
pub async fn update_buffer(_renderer: EntityRef, _id: ResourceId, _data: Vec<u8>) -> CoreResult<()> {
    // Stub - systems/webgl updates buffer data
    Ok(())
}

// Pipeline operations
#[cfg(feature = "pipelines")]
pub async fn create_pipeline(_renderer: EntityRef, _info: crate::resources::PipelineInfo) -> CoreResult<ResourceId> {
    // Stub - systems/webgl creates actual pipeline
    Ok(0)
}

#[cfg(feature = "pipelines")]
pub async fn destroy_pipeline(_renderer: EntityRef, _id: ResourceId) -> CoreResult<()> {
    // Stub - systems/webgl destroys pipeline
    Ok(())
}

#[cfg(feature = "pipelines")]
pub async fn bind_pipeline(_renderer: EntityRef, _id: ResourceId) -> CoreResult<()> {
    // Stub - systems/webgl binds pipeline
    Ok(())
}