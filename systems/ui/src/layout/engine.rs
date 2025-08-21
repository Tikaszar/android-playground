use playground_core_ecs::{World, EntityId};
use playground_core_types::Shared;
use crate::error::{UiError, UiResult};
use crate::element::ElementGraph;
use crate::components::{UiLayoutComponent, LayoutType};
use super::{flexbox::FlexboxLayout, absolute::AbsoluteLayout, docking::DockingLayout};

pub struct LayoutEngine {
    flexbox: FlexboxLayout,
    absolute: AbsoluteLayout,
    docking: DockingLayout,
}

impl LayoutEngine {
    pub fn new() -> Self {
        Self {
            flexbox: FlexboxLayout::new(),
            absolute: AbsoluteLayout::new(),
            docking: DockingLayout::new(),
        }
    }
    
    pub async fn calculate_layout(
        &mut self,
        entity: EntityId,
        graph: &Shared<ElementGraph>,
        world: &Shared<World>,
        screen_size: [f32; 2],
    ) -> UiResult<()> {
        let world_lock = world.read().await;
        let layout = world_lock.get_component::<UiLayoutComponent>(entity).await
            .map_err(|e| UiError::EcsError(e.to_string()))?;
        
        match layout.layout_type {
            LayoutType::Flexbox => {
                self.flexbox.calculate(entity, graph, world, screen_size).await?;
            }
            LayoutType::Absolute => {
                self.absolute.calculate(entity, graph, world, screen_size).await?;
            }
            LayoutType::Docking => {
                self.docking.calculate(entity, graph, world, screen_size).await?;
            }
        }
        
        Ok(())
    }
}