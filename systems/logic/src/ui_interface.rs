use crate::error::{LogicResult, LogicError};
use playground_core_types::{Shared, shared};
use playground_systems_ui::{
    UiSystem, ElementId, ElementStyle, ElementBounds, 
    DiscordLayout, FontWeight, TextAlign
};

/// High-level interface for plugins to interact with the UI system
/// This provides a clean API without exposing internal ECS details
pub struct UiInterface {
    ui_system: Shared<UiSystem>,
}

impl UiInterface {
    pub fn new(ui_system: Shared<UiSystem>) -> Self {
        Self { ui_system }
    }
    
    /// Create a panel element
    pub async fn create_panel(
        &mut self,
        id: &str,
        parent: Option<ElementId>,
    ) -> LogicResult<ElementId> {
        let mut ui = self.ui_system.write().await;
        ui.create_element_with_id(id.to_string(), "panel".to_string(), parent)
            .await
            .map_err(|e| LogicError::SystemError(format!("Failed to create panel: {}", e)))
    }
    
    /// Create a text element
    pub async fn create_text(
        &mut self,
        text: &str,
        parent: Option<ElementId>,
    ) -> LogicResult<ElementId> {
        let mut ui = self.ui_system.write().await;
        let element = ui.create_element("text", parent)
            .await
            .map_err(|e| LogicError::SystemError(format!("Failed to create text: {}", e)))?;
        
        ui.set_element_text(element, text.to_string())
            .await
            .map_err(|e| LogicError::SystemError(format!("Failed to set text: {}", e)))?;
        
        Ok(element)
    }
    
    /// Create a button element
    pub async fn create_button(
        &mut self,
        text: &str,
        parent: Option<ElementId>,
    ) -> LogicResult<ElementId> {
        let mut ui = self.ui_system.write().await;
        let element = ui.create_element("button", parent)
            .await
            .map_err(|e| LogicError::SystemError(format!("Failed to create button: {}", e)))?;
        
        ui.set_element_text(element, text.to_string())
            .await
            .map_err(|e| LogicError::SystemError(format!("Failed to set button text: {}", e)))?;
        
        Ok(element)
    }
    
    /// Style an element
    pub async fn style_element(
        &mut self,
        element: ElementId,
        style: ElementStyle,
    ) -> LogicResult<()> {
        let mut ui = self.ui_system.write().await;
        ui.set_element_style(element, style)
            .await
            .map_err(|e| LogicError::SystemError(format!("Failed to style element: {}", e)))
    }
    
    /// Set element bounds
    pub async fn set_bounds(
        &mut self,
        element: ElementId,
        bounds: ElementBounds,
    ) -> LogicResult<()> {
        let mut ui = self.ui_system.write().await;
        ui.set_element_bounds(element, bounds)
            .await
            .map_err(|e| LogicError::SystemError(format!("Failed to set bounds: {}", e)))
    }
    
    /// Get the root element
    pub async fn get_root(&self) -> Option<ElementId> {
        let ui = self.ui_system.read().await;
        ui.get_root_element()
    }
    
    /// Create a Discord-style layout with all standard components
    pub async fn create_discord_layout(&mut self) -> LogicResult<DiscordLayout> {
        let mut ui = self.ui_system.write().await;
        
        // Get root element
        let root = ui.get_root_element()
            .ok_or_else(|| LogicError::SystemError("No root element".to_string()))?;
        
        // Create sidebar (channel list)
        let sidebar = ui.create_element_with_id(
            "sidebar".to_string(),
            "panel".to_string(),
            Some(root),
        ).await
            .map_err(|e| LogicError::SystemError(format!("Failed to create sidebar: {}", e)))?;
        
        // Style sidebar with Discord colors
        let sidebar_style = ElementStyle {
            background_color: [0.184, 0.192, 0.212, 1.0], // #2f3136
            border_color: [0.125, 0.129, 0.145, 1.0], // #202225
            text_color: [0.863, 0.867, 0.871, 1.0], // #dcddde
            border_width: 0.0,
            border_radius: 0.0,
            opacity: 1.0,
            font_size: 14.0,
            font_family: "Whitney, sans-serif".to_string(),
            font_weight: FontWeight::Normal,
            text_align: TextAlign::Left,
            visible: true,
            z_index: 0,
        };
        ui.set_element_style(sidebar, sidebar_style)
            .await
            .map_err(|e| LogicError::SystemError(format!("Failed to style sidebar: {}", e)))?;
        
        // Set sidebar bounds (240px wide)
        ui.set_element_bounds(sidebar, ElementBounds::new(0.0, 0.0, 240.0, 1080.0))
            .await
            .map_err(|e| LogicError::SystemError(format!("Failed to set sidebar bounds: {}", e)))?;
        
        // Create main content area
        let main_content = ui.create_element_with_id(
            "main-content".to_string(),
            "panel".to_string(),
            Some(root),
        ).await
            .map_err(|e| LogicError::SystemError(format!("Failed to create main content: {}", e)))?;
        
        // Style main content
        let main_style = ElementStyle {
            background_color: [0.212, 0.224, 0.247, 1.0], // #36393f
            border_color: [0.125, 0.129, 0.145, 1.0],
            text_color: [0.863, 0.867, 0.871, 1.0],
            ..Default::default()
        };
        ui.set_element_style(main_content, main_style)
            .await
            .map_err(|e| LogicError::SystemError(format!("Failed to style main content: {}", e)))?;
        
        // Set main content bounds (rest of screen)
        ui.set_element_bounds(main_content, ElementBounds::new(240.0, 0.0, 1680.0, 1080.0))
            .await
            .map_err(|e| LogicError::SystemError(format!("Failed to set main content bounds: {}", e)))?;
        
        // Create message area
        let message_area = ui.create_element_with_id(
            "message-area".to_string(),
            "panel".to_string(),
            Some(main_content),
        ).await
            .map_err(|e| LogicError::SystemError(format!("Failed to create message area: {}", e)))?;
        
        ui.set_element_bounds(message_area, ElementBounds::new(0.0, 0.0, 1680.0, 980.0))
            .await
            .map_err(|e| LogicError::SystemError(format!("Failed to set message area bounds: {}", e)))?;
        
        // Create input area
        let input_area = ui.create_element_with_id(
            "input-area".to_string(),
            "panel".to_string(),
            Some(main_content),
        ).await
            .map_err(|e| LogicError::SystemError(format!("Failed to create input area: {}", e)))?;
        
        ui.set_element_bounds(input_area, ElementBounds::new(0.0, 980.0, 1680.0, 100.0))
            .await
            .map_err(|e| LogicError::SystemError(format!("Failed to set input area bounds: {}", e)))?;
        
        // Mark all as dirty for initial render
        ui.mark_dirty(sidebar).await.ok();
        ui.mark_dirty(main_content).await.ok();
        ui.mark_dirty(message_area).await.ok();
        ui.mark_dirty(input_area).await.ok();
        
        Ok(DiscordLayout {
            sidebar,
            main_content,
            message_area,
            input_area,
            member_list: None,
        })
    }
    
    /// Force a layout update
    pub async fn force_layout(&mut self) -> LogicResult<()> {
        let mut ui = self.ui_system.write().await;
        ui.force_layout()
            .await
            .map_err(|e| LogicError::SystemError(format!("Failed to force layout: {}", e)))
    }
    
    /// Mark an element as dirty for re-render
    pub async fn mark_dirty(&mut self, element: ElementId) -> LogicResult<()> {
        let mut ui = self.ui_system.write().await;
        ui.mark_dirty(element)
            .await
            .map_err(|e| LogicError::SystemError(format!("Failed to mark dirty: {}", e)))
    }
}