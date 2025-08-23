use playground_core_ecs::{World, EntityId};
use playground_core_types::Shared;
use std::sync::Arc;
use crate::error::UiResult;
use crate::element::ElementGraph;
use super::absolute::AbsoluteLayout;

pub struct DockingLayout;

impl DockingLayout {
    pub fn new() -> Self {
        Self
    }
    
    pub async fn calculate(
        &mut self,
        entity: EntityId,
        graph: &Shared<ElementGraph>,
        world: &Arc<World>,
        screen_size: [f32; 2],
    ) -> UiResult<()> {
        // Simplified docking layout
        // For now, just use absolute layout
        AbsoluteLayout::new().calculate(entity, graph, world, screen_size).await
    }
}