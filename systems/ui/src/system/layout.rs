use crate::system::UiSystem;
use crate::error::{UiError, UiResult};
use crate::element::ElementId;
use playground_core_ecs::EntityId;

impl UiSystem {
    pub async fn mark_dirty(&mut self, element: ElementId) -> UiResult<()> {
        self.dirty_elements.write().await.push(element);
        Ok(())
    }
    
    pub async fn force_layout(&mut self) -> UiResult<()> {
        self.update_layout().await
    }
    
    pub(super) async fn update_layout(&mut self) -> UiResult<()> {
        let dirty = self.dirty_elements.read().await.clone();
        
        if dirty.is_empty() {
            return Ok(());
        }
        
        let mut layout_engine = self.layout_engine.write().await;
        
        for entity in dirty {
            layout_engine.calculate_layout(
                entity,
                &self.element_graph,
                &self.world,
                self.screen_size,
            ).await?;
        }
        
        Ok(())
    }
    
    pub(super) async fn mark_subtree_dirty(&self, root: EntityId) -> UiResult<()> {
        let graph = self.element_graph.read().await;
        let mut dirty = self.dirty_elements.write().await;
        
        for element in graph.iter_depth_first(root) {
            dirty.push(element);
        }
        
        Ok(())
    }
}