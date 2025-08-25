use async_trait::async_trait;
use crate::error::{LogicResult, LogicError};
use playground_core_rendering::{RenderCommand, RenderCommandBatch, Viewport};
use playground_core_types::{Shared, shared};
use bytes::Bytes;

/// Interface for rendering that plugins can use without accessing core directly
#[async_trait]
pub trait RenderingInterface: Send + Sync {
    /// Submit render commands to be executed
    async fn submit_commands(&mut self, commands: Vec<RenderCommand>) -> LogicResult<()>;
    
    /// Begin a new frame
    async fn begin_frame(&mut self) -> LogicResult<()>;
    
    /// End the current frame
    async fn end_frame(&mut self) -> LogicResult<()>;
    
    /// Present the rendered frame
    async fn present(&mut self) -> LogicResult<()>;
    
    /// Set the viewport
    async fn set_viewport(&mut self, viewport: Viewport) -> LogicResult<()>;
    
    /// Get current viewport
    fn get_viewport(&self) -> Viewport;
}

/// Concrete renderer data wrapper to avoid dyn trait objects
pub struct RendererData {
    /// Serialized renderer state
    data: Bytes,
    /// Renderer type identifier
    renderer_type: String,
    /// Current viewport
    viewport: Viewport,
}

impl RendererData {
    pub fn new(renderer_type: String) -> Self {
        Self {
            data: Bytes::new(),
            renderer_type,
            viewport: Viewport {
                x: 0,
                y: 0,
                width: 1920,
                height: 1080,
            },
        }
    }
    
    pub fn renderer_type(&self) -> &str {
        &self.renderer_type
    }
    
    pub fn viewport(&self) -> Viewport {
        self.viewport
    }
    
    pub fn set_viewport(&mut self, viewport: Viewport) {
        self.viewport = viewport;
    }
}

/// Concrete implementation that wraps the actual renderer
#[derive(Clone)]
pub struct RendererWrapper {
    renderer_data: Shared<RendererData>,
    frame_commands: Vec<RenderCommand>,
    /// Channel ID for forwarding render commands to the actual renderer
    renderer_channel: u16,
    /// Cached viewport for synchronous access
    cached_viewport: Viewport,
}

impl RendererWrapper {
    pub fn new(renderer_type: String, renderer_channel: u16) -> Self {
        let viewport = Viewport {
            x: 0,
            y: 0,
            width: 1920,
            height: 1080,
        };
        Self {
            renderer_data: shared(RendererData::new(renderer_type)),
            frame_commands: Vec::new(),
            renderer_channel,
            cached_viewport: viewport,
        }
    }
    
    pub fn from_data(renderer_data: Shared<RendererData>, renderer_channel: u16) -> Self {
        let viewport = Viewport {
            x: 0,
            y: 0,
            width: 1920,
            height: 1080,
        };
        Self {
            renderer_data,
            frame_commands: Vec::new(),
            renderer_channel,
            cached_viewport: viewport,
        }
    }
}

#[async_trait]
impl RenderingInterface for RendererWrapper {
    async fn submit_commands(&mut self, commands: Vec<RenderCommand>) -> LogicResult<()> {
        self.frame_commands.extend(commands);
        Ok(())
    }
    
    async fn begin_frame(&mut self) -> LogicResult<()> {
        // In a real implementation, this would send a message to the renderer channel
        // For now, we just track state locally
        Ok(())
    }
    
    async fn end_frame(&mut self) -> LogicResult<()> {
        // Execute all accumulated commands
        if !self.frame_commands.is_empty() {
            let viewport = {
                let data = self.renderer_data.read().await;
                data.viewport()
            };
            
            let mut batch = RenderCommandBatch::new(0);
            batch.set_viewport(viewport);
            
            for command in self.frame_commands.drain(..) {
                batch.push(command);
            }
            
            // In a real implementation, this would send the batch to the renderer channel
            // For now, we just clear the commands
        }
        
        Ok(())
    }
    
    async fn present(&mut self) -> LogicResult<()> {
        // In a real implementation, this would send a present command to the renderer channel
        Ok(())
    }
    
    async fn set_viewport(&mut self, viewport: Viewport) -> LogicResult<()> {
        let mut data = self.renderer_data.write().await;
        data.set_viewport(viewport);
        self.cached_viewport = viewport;
        
        // In a real implementation, this would send a resize command to the renderer channel
        Ok(())
    }
    
    fn get_viewport(&self) -> Viewport {
        self.cached_viewport
    }
}