use playground_core_ecs::{World, EntityId};
use playground_core_types::Shared;
use crate::error::{UiError, UiResult};
use crate::element::ElementGraph;
use crate::components::UiLayoutComponent;

pub struct FlexboxLayout;

impl FlexboxLayout {
    pub fn new() -> Self {
        Self
    }
    
    pub async fn calculate(
        &mut self,
        entity: EntityId,
        graph: &Shared<ElementGraph>,
        world: &Shared<World>,
        screen_size: [f32; 2],
    ) -> UiResult<()> {
        // Simplified flexbox layout
        let mut world_lock = world.write().await;
        let mut layout = world_lock.get_component_mut::<UiLayoutComponent>(entity).await
            .map_err(|e| UiError::EcsError(e.to_string()))?;
        
        // For now, just ensure bounds are set
        if layout.bounds.width == 0.0 {
            layout.bounds.width = screen_size[0];
        }
        if layout.bounds.height == 0.0 {
            layout.bounds.height = screen_size[1];
        }
        
        let padding = layout.padding;
        let bounds = layout.bounds;
        
        drop(world_lock);
        
        // Layout children
        let graph_lock = graph.read().await;
        if let Some(children) = graph_lock.get_children(entity) {
            let mut child_y = bounds.y + padding[0];
            
            for &child in children {
                let mut world_lock = world.write().await;
                let mut child_layout = world_lock.get_component_mut::<UiLayoutComponent>(child).await
                    .map_err(|e| UiError::EcsError(e.to_string()))?;
                
                // Simple vertical layout for now
                child_layout.bounds.x = bounds.x + padding[3];
                child_layout.bounds.y = child_y;
                child_layout.bounds.width = bounds.width - padding[1] - padding[3];
                child_layout.bounds.height = 50.0; // Default height
                
                child_y += child_layout.bounds.height + 10.0; // Spacing
            }
        }
        
        Ok(())
    }
}