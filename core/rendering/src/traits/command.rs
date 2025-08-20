use crate::error::RenderResult;
use crate::batch::RenderCommand;

pub trait CommandEncoder: Send + Sync {
    fn push(&mut self, command: RenderCommand) -> RenderResult<()>;
    
    fn clear(&mut self);
    
    fn commands(&self) -> &[RenderCommand];
    
    fn take_commands(&mut self) -> Vec<RenderCommand>;
}