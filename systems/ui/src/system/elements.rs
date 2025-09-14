use crate::system::UiSystem;
use crate::error::{UiError, UiResult};
use playground_core_ui::{ElementId, ElementType};

impl UiSystem {
    /// Create a new UI element
    pub async fn create_element(
        &mut self,
        element_type: &str,
        parent: Option<ElementId>,
    ) -> UiResult<ElementId> {
        let elem_type = match element_type {
            "panel" => ElementType::Panel,
            "text" => ElementType::Text,
            "button" => ElementType::Button,
            "input" => ElementType::Input,
            "image" => ElementType::Image,
            "scrollview" => ElementType::ScrollView,
            "list" => ElementType::List,
            "grid" => ElementType::Grid,
            "canvas" => ElementType::Canvas,
            _ => ElementType::Custom,
        };
        
        let id = self.storage.create_element(elem_type).await;
        
        // Set parent if provided
        if let Some(parent_id) = parent {
            self.storage.update_element(id, |e| {
                e.parent = Some(parent_id);
            }).await;
            
            self.storage.update_element(parent_id, |e| {
                e.add_child(id);
            }).await;
        }
        
        Ok(id)
    }
    
    /// Remove an element and its children
    pub async fn remove_element(&mut self, id: ElementId) -> UiResult<()> {
        if self.storage.remove_element(id).await {
            Ok(())
        } else {
            Err(UiError::ElementNotFound(format!("{:?}", id)))
        }
    }
    
    /// Update element text
    pub async fn set_element_text(&mut self, id: ElementId, text: String) -> UiResult<()> {
        if self.storage.update_element(id, |e| {
            e.text = Some(text);
        }).await {
            Ok(())
        } else {
            Err(UiError::ElementNotFound(format!("{:?}", id)))
        }
    }
    
    /// Update element visibility
    pub async fn set_element_visible(&mut self, id: ElementId, visible: bool) -> UiResult<()> {
        if self.storage.update_element(id, |e| {
            e.visible = visible;
        }).await {
            Ok(())
        } else {
            Err(UiError::ElementNotFound(format!("{:?}", id)))
        }
    }
}