use crate::system::UiSystem;
use crate::error::UiError;
use async_trait::async_trait;
use playground_core_ui::{
    UiRenderer, UiElementWrapper, ElementId as CoreElementId, ElementType as CoreElementType,
    UiCommand, UiEvent as CoreUiEvent, ElementUpdate, EventResult, Orientation,
    error::{UiResult as CoreUiResult, UiError as CoreUiError},
};
use playground_core_rendering::{RenderCommandBatch, RenderCommand, Viewport};
use uuid::Uuid;

// Implement UiRenderer trait from core/ui
#[async_trait]
impl UiRenderer for UiSystem {
    async fn initialize(&mut self) -> CoreUiResult<()> {
        self.initialize().await
            .map_err(|e| CoreUiError::NotInitialized)
    }
    
    async fn create_element(
        &mut self,
        element_type: CoreElementType,
        parent: Option<CoreElementId>,
    ) -> CoreUiResult<CoreElementId> {
        // Map core element type to our internal type
        let element_type_str = match element_type {
            CoreElementType::Panel => "panel",
            CoreElementType::Text => "text",
            CoreElementType::Button => "button",
            CoreElementType::Input => "input",
            CoreElementType::Image => "image",
            CoreElementType::ScrollView => "scrollview",
            CoreElementType::List => "list",
            CoreElementType::Grid => "grid",
            CoreElementType::Canvas => "canvas",
            CoreElementType::Custom => "custom",
        };
        
        // Convert parent ID if provided
        let parent_entity = if let Some(_parent_id) = parent {
            // For now, we'll need to maintain a mapping between CoreElementId and EntityId
            // This is a simplification - in production you'd have a proper mapping
            None
        } else {
            self.root_entity
        };
        
        let entity = self.create_element(element_type_str, parent_entity).await
            .map_err(|e| CoreUiError::InvalidOperation(e.to_string()))?;
        
        // Create a CoreElementId from the entity
        Ok(CoreElementId(Uuid::new_v4()))
    }
    
    async fn update_element(
        &mut self,
        id: CoreElementId,
        update: ElementUpdate,
    ) -> CoreUiResult<()> {
        // This would need proper ID mapping in production
        match update {
            ElementUpdate::Text(text) => {
                // Find entity and update text
                Ok(())
            }
            ElementUpdate::Style(style) => {
                // Convert CoreStyle to our internal style and apply
                Ok(())
            }
            ElementUpdate::Bounds(bounds) => {
                // Update element bounds
                Ok(())
            }
            _ => Ok(())
        }
    }
    
    async fn remove_element(&mut self, id: CoreElementId) -> CoreUiResult<()> {
        // Find and remove element
        Ok(())
    }
    
    async fn get_element(&self, id: CoreElementId) -> CoreUiResult<UiElementWrapper> {
        // Return a wrapper instead of dyn trait object
        // For now, return error since element retrieval isn't fully implemented
        Err(CoreUiError::ElementNotFound(format!("{:?}", id)))
    }
    
    async fn process_command(&mut self, command: UiCommand) -> CoreUiResult<()> {
        match command {
            UiCommand::CreateElement { id, element_type, parent } => {
                // Map core element type to our internal type
                let element_type_str = match element_type {
                    CoreElementType::Panel => "panel",
                    CoreElementType::Text => "text",
                    CoreElementType::Button => "button",
                    CoreElementType::Input => "input",
                    CoreElementType::Image => "image",
                    CoreElementType::ScrollView => "scrollview",
                    CoreElementType::List => "list",
                    CoreElementType::Grid => "grid",
                    CoreElementType::Canvas => "canvas",
                    CoreElementType::Custom => "custom",
                };
                
                // Convert parent CoreElementId to internal EntityId (would need mapping)
                let parent_entity = self.root_entity;
                self.create_element(element_type_str, parent_entity).await
                    .map_err(|e| CoreUiError::InvalidOperation(e.to_string()))?;
                Ok(())
            }
            UiCommand::SetText { id, text } => {
                self.update_element(id, ElementUpdate::Text(text)).await
            }
            _ => Ok(())
        }
    }
    
    async fn handle_event(&mut self, event: CoreUiEvent) -> CoreUiResult<EventResult> {
        // Convert core event to our internal event and handle
        Ok(EventResult::Ignored)
    }
    
    async fn calculate_layout(&mut self) -> CoreUiResult<()> {
        self.update_layout().await
            .map_err(|e| CoreUiError::LayoutFailed(e.to_string()))
    }
    
    async fn render_frame(&mut self, frame_id: u64) -> CoreUiResult<RenderCommandBatch> {
        self.frame_id = frame_id;
        
        // Create render command batch
        let mut batch = RenderCommandBatch::new(frame_id);
        
        // Clear with mobile-friendly dark background (Discord-like)
        batch.push(RenderCommand::Clear {
            color: [0.133, 0.137, 0.153, 1.0],
        });
        
        // Render the UI tree
        if let Some(root) = self.root_entity {
            let theme_mgr = self.theme_manager.read().await;
            let theme = theme_mgr.get_theme(self.current_theme)
                .map_err(|e| CoreUiError::RenderingFailed(e.to_string()))?
                .clone();
            drop(theme_mgr);
            
            self.render_element_tree(root, &mut batch, &theme).await
                .map_err(|e| CoreUiError::RenderingFailed(e.to_string()))?;
        }
        
        Ok(batch)
    }
    
    async fn get_root(&self) -> Option<CoreElementId> {
        // Would need proper ID mapping
        self.root_entity.map(|_| CoreElementId(Uuid::new_v4()))
    }
    
    async fn set_viewport(&mut self, width: f32, height: f32) -> CoreUiResult<()> {
        self.viewport = Viewport {
            x: 0,
            y: 0,
            width: width as u32,
            height: height as u32,
        };
        self.screen_size = [width, height];
        
        // Update root element bounds
        if let Some(_root) = self.root_entity {
            // TODO: Update root layout bounds to match viewport
        }
        
        Ok(())
    }
    
    async fn set_safe_area_insets(
        &mut self,
        top: f32,
        bottom: f32,
        left: f32,
        right: f32,
    ) -> CoreUiResult<()> {
        // Store safe area insets for mobile layout
        let mut mobile = self.mobile_features.write().await;
        mobile.set_safe_area_insets(top, bottom, left, right).await
            .map_err(|e| CoreUiError::InvalidOperation(e.to_string()))?;
        Ok(())
    }
    
    async fn handle_orientation_change(&mut self, orientation: Orientation) -> CoreUiResult<()> {
        // Handle mobile orientation change
        let mut mobile = self.mobile_features.write().await;
        
        // Update screen size based on orientation
        match orientation {
            Orientation::Portrait | Orientation::PortraitUpsideDown => {
                // Portrait mode - taller than wide
                if self.screen_size[0] > self.screen_size[1] {
                    self.screen_size = [self.screen_size[1], self.screen_size[0]];
                }
            }
            Orientation::LandscapeLeft | Orientation::LandscapeRight => {
                // Landscape mode - wider than tall
                if self.screen_size[1] > self.screen_size[0] {
                    self.screen_size = [self.screen_size[1], self.screen_size[0]];
                }
            }
        }
        
        // Mark all elements as dirty for re-layout
        if let Some(root) = self.root_entity {
            self.mark_subtree_dirty(root).await
                .map_err(|e| CoreUiError::LayoutFailed(e.to_string()))?;
        }
        
        Ok(())
    }
}