use playground_core_ecs::{World, EntityId};
use playground_core_types::Shared;
use crate::error::{UiError, UiResult};
use crate::element::ElementGraph;
use crate::components::*;

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

struct FlexboxLayout;

impl FlexboxLayout {
    fn new() -> Self {
        Self
    }
    
    async fn calculate(
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
        
        drop(world_lock);
        
        // Layout children
        let graph_lock = graph.read().await;
        if let Some(children) = graph_lock.get_children(entity) {
            let mut child_y = layout.bounds.y + layout.padding[0];
            
            for &child in children {
                let mut world_lock = world.write().await;
                let mut child_layout = world_lock.get_component_mut::<UiLayoutComponent>(child).await
                    .map_err(|e| UiError::EcsError(e.to_string()))?;
                
                // Simple vertical layout for now
                child_layout.bounds.x = layout.bounds.x + layout.padding[3];
                child_layout.bounds.y = child_y;
                child_layout.bounds.width = layout.bounds.width - layout.padding[1] - layout.padding[3];
                child_layout.bounds.height = 50.0; // Default height
                
                child_y += child_layout.bounds.height + 10.0; // Spacing
            }
        }
        
        Ok(())
    }
}

struct AbsoluteLayout;

impl AbsoluteLayout {
    fn new() -> Self {
        Self
    }
    
    async fn calculate(
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

struct DockingLayout;

impl DockingLayout {
    fn new() -> Self {
        Self
    }
    
    async fn calculate(
        &mut self,
        entity: EntityId,
        graph: &Shared<ElementGraph>,
        world: &Shared<World>,
        screen_size: [f32; 2],
    ) -> UiResult<()> {
        // Simplified docking layout
        let world_lock = world.read().await;
        let layout = world_lock.get_component::<UiLayoutComponent>(entity).await
            .map_err(|e| UiError::EcsError(e.to_string()))?;
        
        // For now, just use absolute layout
        drop(world_lock);
        
        AbsoluteLayout::new().calculate(entity, graph, world, screen_size).await
    }
}