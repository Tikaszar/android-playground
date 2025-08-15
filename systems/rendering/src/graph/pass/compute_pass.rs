use crate::graph::pass::{Pass, PassId};
use crate::commands::CommandBuffer;
use crate::error::RendererError;

pub struct ComputePass {
    name: String,
    dependencies: Vec<PassId>,
}

impl ComputePass {
    pub fn new(name: String) -> Self {
        Self {
            name,
            dependencies: Vec::new(),
        }
    }
    
    pub fn with_dependencies(mut self, deps: Vec<PassId>) -> Self {
        self.dependencies = deps;
        self
    }
}

impl Pass for ComputePass {
    fn name(&self) -> &str {
        &self.name
    }
    
    fn dependencies(&self) -> &[PassId] {
        &self.dependencies
    }
    
    fn execute(&mut self, _encoder: &mut dyn CommandBuffer) -> Result<(), RendererError> {
        #[cfg(feature = "webgl")]
        return Err(RendererError::NotSupported("Compute shaders not supported in WebGL".to_string()));
        
        #[cfg(not(feature = "webgl"))]
        Ok(())
    }
}