use crate::graph::pass::{Pass, PassId};
use std::collections::HashMap;

pub struct RenderGraph {
    passes: HashMap<PassId, Box<dyn Pass>>,
    execution_order: Vec<PassId>,
}

impl RenderGraph {
    pub fn new() -> Self {
        Self {
            passes: HashMap::new(),
            execution_order: Vec::new(),
        }
    }
    
    pub fn add_pass(&mut self, pass: Box<dyn Pass>) -> PassId {
        let id = PassId::new();
        self.passes.insert(id, pass);
        self.execution_order.push(id);
        id
    }
    
    pub fn remove_pass(&mut self, id: PassId) -> Option<Box<dyn Pass>> {
        self.execution_order.retain(|&pass_id| pass_id != id);
        self.passes.remove(&id)
    }
    
    pub fn get_pass(&self, id: PassId) -> Option<&dyn Pass> {
        self.passes.get(&id).map(|p| p.as_ref())
    }
    
    pub fn get_pass_mut(&mut self, id: PassId) -> Option<&mut dyn Pass> {
        self.passes.get_mut(&id).map(|p| p.as_mut() as &mut dyn Pass)
    }
    
    pub fn execution_order(&self) -> &[PassId] {
        &self.execution_order
    }
    
    pub fn set_execution_order(&mut self, order: Vec<PassId>) {
        self.execution_order = order;
    }
}