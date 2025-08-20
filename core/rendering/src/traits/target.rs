use async_trait::async_trait;
use crate::error::RenderResult;

#[async_trait]
pub trait RenderTarget: Send + Sync {
    async fn bind(&mut self) -> RenderResult<()>;
    
    async fn unbind(&mut self) -> RenderResult<()>;
    
    async fn clear(&mut self, color: [f32; 4]) -> RenderResult<()>;
    
    async fn resize(&mut self, width: u32, height: u32) -> RenderResult<()>;
    
    fn width(&self) -> u32;
    
    fn height(&self) -> u32;
    
    fn handle(&self) -> u32;
}