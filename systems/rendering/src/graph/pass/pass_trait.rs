use crate::commands::CommandBuffer;
use crate::error::RendererError;

pub trait Pass: Send + Sync {
    fn name(&self) -> &str;
    fn dependencies(&self) -> &[crate::graph::PassId];
    fn execute(&mut self, encoder: &mut dyn CommandBuffer) -> Result<(), RendererError>;
}