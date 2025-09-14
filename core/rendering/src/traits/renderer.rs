use async_trait::async_trait;
use crate::error::RenderResult;
use crate::batch::RenderCommandBatch;
use crate::wrapper::RenderTargetWrapper;

#[async_trait]
pub trait Renderer: Send + Sync {
    async fn initialize(&mut self) -> RenderResult<()>;
    
    async fn begin_frame(&mut self) -> RenderResult<()>;
    
    async fn execute_commands(&mut self, batch: &RenderCommandBatch) -> RenderResult<()>;
    
    async fn end_frame(&mut self) -> RenderResult<()>;
    
    async fn present(&mut self) -> RenderResult<()>;
    
    async fn resize(&mut self, width: u32, height: u32) -> RenderResult<()>;
    
    async fn create_render_target(&mut self, width: u32, height: u32) -> RenderResult<RenderTargetWrapper>;
    
    async fn shutdown(&mut self) -> RenderResult<()>;
    
    fn capabilities(&self) -> RendererCapabilities;
    
    fn is_initialized(&self) -> bool;
}

#[derive(Debug, Clone)]
pub struct RendererCapabilities {
    pub max_texture_size: u32,
    pub max_render_targets: u32,
    pub supports_compute: bool,
    pub supports_instancing: bool,
    pub supports_tessellation: bool,
    pub max_vertex_attributes: u32,
    pub max_uniform_buffer_size: usize,
}