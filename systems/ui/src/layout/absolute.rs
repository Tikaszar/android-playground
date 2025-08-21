use playground_core_ecs::{World, EntityId};
use playground_core_types::Shared;
use crate::error::{UiError, UiResult};
use crate::element::ElementGraph;
use crate::components::UiLayoutComponent;

pub struct AbsoluteLayout;

impl AbsoluteLayout {
    pub fn new() -> Self {
        Self
    }
    
    pub async fn calculate(
        &mut self,
        entity: EntityId,
        _graph: &Shared<ElementGraph>,
        world: &Shared<World>,
        screen_size: [f32; 2],
    ) -> UiResult<()> {
        // Absolute positioning - elements use their set positions
        let mut world_lock = world.write().await;
        let mut layout = world_lock.get_component_mut::<UiLayoutComponent>(entity).await
            .map_err(|e| UiError::EcsError(e.to_string()))?;
        
        // Ensure bounds are within screen
        if layout.bounds.x + layout.bounds.width > screen_size[0] {
            layout.bounds.width = screen_size[0] - layout.bounds.x;
        }
        if layout.bounds.y + layout.bounds.height > screen_size[1] {
            layout.bounds.height = screen_size[1] - layout.bounds.y;
        }
        
        Ok(())
    }
}