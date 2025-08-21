use async_trait::async_trait;
use crate::error::{LogicResult, LogicError};
use playground_core_rendering::{RenderCommand, RenderCommandBatch, Viewport};
use playground_core_types::Shared;

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

/// Concrete implementation that wraps the actual renderer
pub struct RendererWrapper {
    renderer: Shared<Box<dyn playground_core_rendering::Renderer>>,
    viewport: Viewport,
    frame_commands: Vec<RenderCommand>,
}

impl RendererWrapper {
    pub fn new(renderer: Shared<Box<dyn playground_core_rendering::Renderer>>) -> Self {
        Self {
            renderer,
            viewport: Viewport {
                x: 0,
                y: 0,
                width: 1920,
                height: 1080,
            },
            frame_commands: Vec::new(),
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
        let mut renderer = self.renderer.write().await;
        renderer.begin_frame()
            .await
            .map_err(|e| LogicError::SystemError(format!("Failed to begin frame: {}", e)))?;
        Ok(())
    }
    
    async fn end_frame(&mut self) -> LogicResult<()> {
        // Execute all accumulated commands
        if !self.frame_commands.is_empty() {
            let mut batch = RenderCommandBatch::new(0);
            batch.set_viewport(self.viewport);
            
            for command in self.frame_commands.drain(..) {
                batch.push(command);
            }
            
            let mut renderer = self.renderer.write().await;
            renderer.execute_commands(&batch)
                .await
                .map_err(|e| LogicError::SystemError(format!("Failed to execute commands: {}", e)))?;
        }
        
        let mut renderer = self.renderer.write().await;
        renderer.end_frame()
            .await
            .map_err(|e| LogicError::SystemError(format!("Failed to end frame: {}", e)))?;
        Ok(())
    }
    
    async fn present(&mut self) -> LogicResult<()> {
        let mut renderer = self.renderer.write().await;
        renderer.present()
            .await
            .map_err(|e| LogicError::SystemError(format!("Failed to present: {}", e)))?;
        Ok(())
    }
    
    async fn set_viewport(&mut self, viewport: Viewport) -> LogicResult<()> {
        self.viewport = viewport;
        let mut renderer = self.renderer.write().await;
        renderer.resize(viewport.width as u32, viewport.height as u32)
            .await
            .map_err(|e| LogicError::SystemError(format!("Failed to resize: {}", e)))?;
        Ok(())
    }
    
    fn get_viewport(&self) -> Viewport {
        self.viewport
    }
}