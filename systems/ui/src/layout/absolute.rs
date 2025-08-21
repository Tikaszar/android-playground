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
        let world_lock = world.write().await;
        let layout = world_lock.get_component::<UiLayoutComponent>(entity).await
            .map_err(|e| UiError::EcsError(e.to_string()))?;
        
        let mut bounds = layout.bounds;
        let mut needs_update = false;
        
        // Ensure bounds are within screen
        if bounds.x + bounds.width > screen_size[0] {
            bounds.width = screen_size[0] - bounds.x;
            needs_update = true;
        }
        if bounds.y + bounds.height > screen_size[1] {
            bounds.height = screen_size[1] - bounds.y;
            needs_update = true;
        }
        
        // Update if needed
        if needs_update {
            world_lock.update_component::<UiLayoutComponent>(entity, |l| {
                l.bounds = bounds;
            }).await.map_err(|e| UiError::EcsError(e.to_string()))?;
        }
        
        Ok(())
    }
}