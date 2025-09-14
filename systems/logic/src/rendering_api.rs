//! Rendering API module for systems/logic
//! 
//! This provides the public API for rendering operations.
//! All functions forward to renderer command processors.

use playground_core_rendering::{RenderCommand, RenderCommandBatch};
use playground_core_ecs::EcsResult;
use bytes::Bytes;

/// Submit a batch of render commands
pub async fn render_batch(commands: Vec<RenderCommand>) -> EcsResult<()> {
    let batch = RenderCommandBatch { commands };
    
    let payload = serde_json::to_vec(&batch)
        .map_err(|e| playground_core_ecs::EcsError::Generic(e.to_string()))?;
    
    playground_core_ecs::system_command_access::send_to_system(
        "webgl_renderer",
        "render_batch",
        Bytes::from(payload)
    ).await?;
    
    Ok(())
}

/// Clear the screen with a color
pub async fn clear_screen(color: [f32; 4]) -> EcsResult<()> {
    render_batch(vec![
        RenderCommand::Clear {
            color,
            depth: true,
            stencil: false,
        }
    ]).await
}

/// Resize the renderer viewport
pub async fn resize_viewport(width: u32, height: u32) -> EcsResult<()> {
    #[derive(serde::Serialize)]
    struct ResizeData {
        width: u32,
        height: u32,
    }
    
    let data = ResizeData { width, height };
    let payload = serde_json::to_vec(&data)
        .map_err(|e| playground_core_ecs::EcsError::Generic(e.to_string()))?;
    
    playground_core_ecs::system_command_access::send_to_system(
        "webgl_renderer",
        "resize",
        Bytes::from(payload)
    ).await?;
    
    Ok(())
}

/// Draw a simple quad
pub async fn draw_quad(
    position: [f32; 2],
    size: [f32; 2],
    color: [f32; 4]
) -> EcsResult<()> {
    render_batch(vec![
        RenderCommand::DrawQuad {
            position,
            size,
            color,
        }
    ]).await
}

/// Draw text
pub async fn draw_text(
    text: String,
    position: [f32; 2],
    size: f32,
    color: [f32; 4]
) -> EcsResult<()> {
    render_batch(vec![
        RenderCommand::DrawText {
            text,
            position,
            size,
            color,
            font: None,
        }
    ]).await
}

/// Draw a line
pub async fn draw_line(
    start: [f32; 2],
    end: [f32; 2],
    width: f32,
    color: [f32; 4]
) -> EcsResult<()> {
    render_batch(vec![
        RenderCommand::DrawLine {
            start,
            end,
            width,
            color,
        }
    ]).await
}

/// Draw a circle
pub async fn draw_circle(
    center: [f32; 2],
    radius: f32,
    color: [f32; 4],
    filled: bool
) -> EcsResult<()> {
    render_batch(vec![
        RenderCommand::DrawCircle {
            center,
            radius,
            color,
            filled,
        }
    ]).await
}

/// Draw an image/texture
pub async fn draw_image(
    texture_id: u32,
    position: [f32; 2],
    size: [f32; 2],
    source_rect: Option<[f32; 4]>
) -> EcsResult<()> {
    render_batch(vec![
        RenderCommand::DrawImage {
            texture_id,
            position,
            size,
            source_rect,
            tint: [1.0, 1.0, 1.0, 1.0],
        }
    ]).await
}

/// Set clipping rectangle
pub async fn set_clip_rect(rect: [f32; 4]) -> EcsResult<()> {
    render_batch(vec![
        RenderCommand::SetClipRect { rect }
    ]).await
}

/// Clear clipping
pub async fn clear_clip() -> EcsResult<()> {
    render_batch(vec![
        RenderCommand::ClearClipRect
    ]).await
}

/// Push a transform onto the stack
pub async fn push_transform(transform: [[f32; 3]; 3]) -> EcsResult<()> {
    render_batch(vec![
        RenderCommand::PushTransform { transform }
    ]).await
}

/// Pop a transform from the stack
pub async fn pop_transform() -> EcsResult<()> {
    render_batch(vec![
        RenderCommand::PopTransform
    ]).await
}

/// Begin a new frame
pub async fn begin_frame() -> EcsResult<()> {
    render_batch(vec![
        RenderCommand::BeginFrame
    ]).await
}

/// End the current frame
pub async fn end_frame() -> EcsResult<()> {
    render_batch(vec![
        RenderCommand::EndFrame
    ]).await
}

/// Present the rendered frame
pub async fn present_frame() -> EcsResult<()> {
    render_batch(vec![
        RenderCommand::Present
    ]).await
}

/// Get renderer capabilities
pub async fn get_renderer_capabilities() -> EcsResult<RendererCapabilities> {
    // Would query the renderer
    Ok(RendererCapabilities {
        max_texture_size: 16384,
        max_render_targets: 16,
        supports_instancing: true,
        supports_tessellation: false,
    })
}

/// Renderer capabilities
#[derive(Debug, Clone)]
pub struct RendererCapabilities {
    pub max_texture_size: u32,
    pub max_render_targets: u32,
    pub supports_instancing: bool,
    pub supports_tessellation: bool,
}