use crate::graph::pass::{Pass, PassId};
use crate::commands::CommandBuffer;
use crate::error::RendererError;
use crate::resources::RenderTargetHandle;

pub struct RenderPass {
    name: String,
    dependencies: Vec<PassId>,
    render_target: RenderTargetHandle,
}

impl RenderPass {
    pub fn new(name: String, render_target: RenderTargetHandle) -> Self {
        Self {
            name,
            dependencies: Vec::new(),
            render_target,
        }
    }
    
    pub fn with_dependencies(mut self, deps: Vec<PassId>) -> Self {
        self.dependencies = deps;
        self
    }
}

impl Pass for RenderPass {
    fn name(&self) -> &str {
        &self.name
    }
    
    fn dependencies(&self) -> &[PassId] {
        &self.dependencies
    }
    
    fn execute(&mut self, encoder: &mut dyn CommandBuffer) -> Result<(), RendererError> {
        encoder.set_render_target(self.render_target);
        Ok(())
    }
}