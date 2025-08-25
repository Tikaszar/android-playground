use playground_core_ecs::{World, EntityId};
use playground_core_types::{Shared, Handle};
use crate::error::{UiError, UiResult};
use crate::element::ElementGraph;
use crate::components::{UiLayoutComponent, ElementBounds};

pub struct FlexboxLayout;

impl FlexboxLayout {
    pub fn new() -> Self {
        Self
    }
    
    pub async fn calculate(
        &mut self,
        entity: EntityId,
        graph: &Shared<ElementGraph>,
        world: &Handle<World>,
        screen_size: [f32; 2],
    ) -> UiResult<()> {
        // Simplified flexbox layout - world is Handle<World> now
        // Get the current layout for reading
        let layout = world.get_component::<UiLayoutComponent>(entity).await
            .map_err(|e| UiError::EcsError(e.to_string()))?;
        
        let mut bounds = layout.bounds;
        let padding = layout.padding;
        
        // Update bounds if needed
        let needs_update = bounds.width == 0.0 || bounds.height == 0.0;
        if bounds.width == 0.0 {
            bounds.width = screen_size[0];
        }
        if bounds.height == 0.0 {
            bounds.height = screen_size[1];
        }
        
        // Update the component if bounds changed
        if needs_update {
            world.update_component::<UiLayoutComponent>(entity, |l| {
                l.bounds = bounds;
            }).await.map_err(|e| UiError::EcsError(e.to_string()))?;
        }
        
        // Layout children
        let graph_lock = graph.read().await;
        if let Some(children) = graph_lock.get_children(entity) {
            let mut child_y = bounds.y + padding[0];
            
            for &child in children {
                // Calculate new bounds for child
                let new_bounds = ElementBounds {
                    x: bounds.x + padding[3],
                    y: child_y,
                    width: bounds.width - padding[1] - padding[3],
                    height: 50.0, // Default height
                };
                
                // Update child layout - world is Handle<World> now
                world.update_component::<UiLayoutComponent>(child, |layout| {
                    layout.bounds = new_bounds;
                }).await.map_err(|e| UiError::EcsError(e.to_string()))?;
                
                child_y += new_bounds.height + 10.0; // Spacing
            }
        }
        
        Ok(())
    }
}