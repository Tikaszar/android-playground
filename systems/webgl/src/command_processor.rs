//! WebGL renderer command processor for handling rendering commands through ECS

use async_trait::async_trait;
use playground_core_ecs::{
    SystemCommandProcessor, SystemResponse, EcsResult, EcsError
};
use playground_core_rendering::{RenderCommand, RenderCommandBatch};
use crate::renderer::WebGLRenderer;
use bytes::Bytes;
use serde::{Serialize, Deserialize};
use serde_json;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Renderer-specific commands
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RendererCommand {
    /// Render a batch of commands
    RenderBatch(RenderCommandBatch),
    /// Resize the viewport
    Resize { width: u32, height: u32 },
    /// Clear the screen
    Clear { color: [f32; 4] },
    /// Present the rendered frame
    Present,
    /// Get renderer capabilities
    GetCapabilities,
    /// Get current viewport size
    GetViewport,
    /// Set viewport
    SetViewport { x: u32, y: u32, width: u32, height: u32 },
    /// Enable/disable vsync
    SetVSync { enabled: bool },
    /// Get frame statistics
    GetFrameStats,
}

/// Renderer command responses
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RendererResponse {
    Success,
    Capabilities {
        max_texture_size: u32,
        max_render_targets: u32,
        supports_instancing: bool,
        supports_tessellation: bool,
    },
    Viewport {
        x: u32,
        y: u32,
        width: u32,
        height: u32,
    },
    FrameStats {
        frames_rendered: u64,
        commands_processed: u64,
        draw_calls: u64,
        vertices_rendered: u64,
    },
}

/// WebGL renderer command processor
pub struct RendererCommandProcessor {
    renderer: Arc<RwLock<WebGLRenderer>>,
}

impl RendererCommandProcessor {
    pub fn new(renderer: Arc<RwLock<WebGLRenderer>>) -> Self {
        Self { renderer }
    }
}

#[async_trait]
impl SystemCommandProcessor for RendererCommandProcessor {
    fn system_name(&self) -> &str {
        "webgl_renderer"
    }
    
    async fn handle_system_command(&self, command_type: &str, payload: Bytes) -> EcsResult<SystemResponse> {
        match command_type {
            "renderer_command" => {
                // Deserialize renderer command
                let command: RendererCommand = serde_json::from_slice(&payload)
                    .map_err(|e| EcsError::Generic(format!("Failed to deserialize renderer command: {}", e)))?;
                
                // Process the command
                let result = self.process_renderer_command(command).await;
                
                match result {
                    Ok(response) => {
                        let response_bytes = serde_json::to_vec(&response)
                            .map_err(|e| EcsError::Generic(e.to_string()))?;
                        
                        Ok(SystemResponse {
                            success: true,
                            payload: Some(Bytes::from(response_bytes)),
                            error: None,
                        })
                    },
                    Err(e) => {
                        Ok(SystemResponse {
                            success: false,
                            payload: None,
                            error: Some(e.to_string()),
                        })
                    }
                }
            },
            "render_batch" => {
                // Direct render batch command
                let batch: RenderCommandBatch = serde_json::from_slice(&payload)
                    .map_err(|e| EcsError::Generic(format!("Failed to deserialize render batch: {}", e)))?;
                
                let mut renderer = self.renderer.write().await;
                renderer.render_batch(&batch.commands).await
                    .map_err(|e| EcsError::Generic(e.to_string()))?;
                
                Ok(SystemResponse {
                    success: true,
                    payload: None,
                    error: None,
                })
            },
            "resize" => {
                // Direct resize command
                #[derive(Deserialize)]
                struct ResizeData { width: u32, height: u32 }
                
                let data: ResizeData = serde_json::from_slice(&payload)
                    .map_err(|e| EcsError::Generic(format!("Failed to deserialize resize data: {}", e)))?;
                
                let mut renderer = self.renderer.write().await;
                renderer.resize(data.width, data.height).await
                    .map_err(|e| EcsError::Generic(e.to_string()))?;
                
                Ok(SystemResponse {
                    success: true,
                    payload: None,
                    error: None,
                })
            },
            _ => {
                Err(EcsError::Generic(format!("Unknown renderer command type: {}", command_type)))
            }
        }
    }
    
    fn supported_commands(&self) -> Vec<String> {
        vec![
            "renderer_command".to_string(),
            "render_batch".to_string(),
            "resize".to_string(),
        ]
    }
}

impl RendererCommandProcessor {
    async fn process_renderer_command(&self, command: RendererCommand) -> Result<RendererResponse, EcsError> {
        match command {
            RendererCommand::RenderBatch(batch) => {
                let mut renderer = self.renderer.write().await;
                renderer.render_batch(&batch.commands).await
                    .map_err(|e| EcsError::Generic(e.to_string()))?;
                Ok(RendererResponse::Success)
            },
            RendererCommand::Resize { width, height } => {
                let mut renderer = self.renderer.write().await;
                renderer.resize(width, height).await
                    .map_err(|e| EcsError::Generic(e.to_string()))?;
                Ok(RendererResponse::Success)
            },
            RendererCommand::Clear { color } => {
                let mut renderer = self.renderer.write().await;
                // Use clear command
                renderer.render_batch(&[RenderCommand::Clear { 
                    color, 
                    depth: true, 
                    stencil: false 
                }]).await
                    .map_err(|e| EcsError::Generic(e.to_string()))?;
                Ok(RendererResponse::Success)
            },
            RendererCommand::Present => {
                // WebGL automatically presents at frame end
                Ok(RendererResponse::Success)
            },
            RendererCommand::GetCapabilities => {
                let renderer = self.renderer.read().await;
                let caps = renderer.capabilities();
                Ok(RendererResponse::Capabilities {
                    max_texture_size: caps.max_texture_size,
                    max_render_targets: caps.max_render_targets,
                    supports_instancing: caps.supports_instancing,
                    supports_tessellation: caps.supports_tessellation,
                })
            },
            RendererCommand::GetViewport => {
                // Would need to access viewport from renderer
                // For now, return default
                Ok(RendererResponse::Viewport {
                    x: 0,
                    y: 0,
                    width: 1920,
                    height: 1080,
                })
            },
            RendererCommand::SetViewport { x, y, width, height } => {
                // Would need to add viewport setting to renderer
                // For now, just succeed
                Ok(RendererResponse::Success)
            },
            RendererCommand::SetVSync { enabled: _ } => {
                // VSync is handled by browser in WebGL
                Ok(RendererResponse::Success)
            },
            RendererCommand::GetFrameStats => {
                // Would need to track stats in renderer
                // For now, return dummy stats
                Ok(RendererResponse::FrameStats {
                    frames_rendered: 0,
                    commands_processed: 0,
                    draw_calls: 0,
                    vertices_rendered: 0,
                })
            },
        }
    }
}