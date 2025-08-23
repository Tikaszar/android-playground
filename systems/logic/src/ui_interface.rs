use crate::error::{LogicResult, LogicError};
use playground_core_types::{Shared, shared};
use playground_core_ui::{
    UiRenderer, ElementId as CoreElementId, ElementType, ElementUpdate,
    Style, Bounds, UiCommand, UiEvent, EventResult
};
use playground_systems_ui::{
    UiSystem, ElementId, ElementStyle, ElementBounds, 
    DiscordLayout, FontWeight, TextAlign
};
use async_trait::async_trait;
use std::sync::Arc;

/// High-level interface for plugins to interact with the UI system
/// This provides a clean API without exposing internal ECS details
pub struct UiInterface {
    ui_system: Shared<UiSystem>,
    // Track mapping between core ElementIds and internal ElementIds
    element_mapping: Shared<std::collections::HashMap<CoreElementId, ElementId>>,
    systems_manager: Option<Arc<crate::SystemsManager>>,
}

impl UiInterface {
    pub fn new(ui_system: Shared<UiSystem>) -> Self {
        Self { 
            ui_system,
            element_mapping: shared(std::collections::HashMap::new()),
            systems_manager: None,
        }
    }
    
    pub fn with_systems_manager(ui_system: Shared<UiSystem>, systems_manager: Arc<crate::SystemsManager>) -> Self {
        Self { 
            ui_system,
            element_mapping: shared(std::collections::HashMap::new()),
            systems_manager: Some(systems_manager),
        }
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
    
    // Mobile-specific Discord UI methods
    
    /// Create a mobile Discord channel list
    pub async fn create_mobile_channel_list(&mut self, parent: Option<ElementId>) -> LogicResult<ElementId> {
        let mut ui = self.ui_system.write().await;
        
        // Create scrollable container for channels
        let container = ui.create_element_with_id(
            "channel-list".to_string(),
            "scrollview".to_string(),
            parent,
        ).await
            .map_err(|e| LogicError::SystemError(format!("Failed to create channel list: {}", e)))?;
        
        // Style for mobile channel list - full width, touch-friendly
        let style = ElementStyle {
            background_color: [0.184, 0.192, 0.212, 1.0], // Discord sidebar color
            border_color: [0.125, 0.129, 0.145, 1.0],
            text_color: [0.863, 0.867, 0.871, 1.0],
            border_width: 0.0,
            border_radius: 0.0,
            opacity: 1.0,
            font_size: 16.0, // Larger for mobile
            font_family: "system-ui, sans-serif".to_string(),
            font_weight: FontWeight::Normal,
            text_align: TextAlign::Left,
            visible: true,
            z_index: 10,
        };
        
        ui.set_element_style(container, style).await
            .map_err(|e| LogicError::SystemError(format!("Failed to style channel list: {}", e)))?;
        
        Ok(container)
    }
    
    /// Create a mobile Discord message area
    pub async fn create_mobile_message_area(&mut self, parent: Option<ElementId>) -> LogicResult<ElementId> {
        let mut ui = self.ui_system.write().await;
        
        let message_area = ui.create_element_with_id(
            "message-area".to_string(),
            "scrollview".to_string(),
            parent,
        ).await
            .map_err(|e| LogicError::SystemError(format!("Failed to create message area: {}", e)))?;
        
        // Mobile-optimized message area style
        let style = ElementStyle {
            background_color: [0.212, 0.224, 0.247, 1.0], // Discord main area color
            border_color: [0.0, 0.0, 0.0, 0.0],
            text_color: [0.863, 0.867, 0.871, 1.0],
            border_width: 0.0,
            border_radius: 0.0,
            opacity: 1.0,
            font_size: 15.0, // Comfortable mobile reading size
            font_family: "system-ui, sans-serif".to_string(),
            font_weight: FontWeight::Normal,
            text_align: TextAlign::Left,
            visible: true,
            z_index: 0,
        };
        
        ui.set_element_style(message_area, style).await
            .map_err(|e| LogicError::SystemError(format!("Failed to style message area: {}", e)))?;
        
        Ok(message_area)
    }
    
    /// Create a mobile Discord input bar
    pub async fn create_mobile_input_bar(&mut self, parent: Option<ElementId>) -> LogicResult<ElementId> {
        let mut ui = self.ui_system.write().await;
        
        let input_bar = ui.create_element_with_id(
            "input-bar".to_string(),
            "panel".to_string(),
            parent,
        ).await
            .map_err(|e| LogicError::SystemError(format!("Failed to create input bar: {}", e)))?;
        
        // Mobile input bar with larger touch targets
        let style = ElementStyle {
            background_color: [0.251, 0.263, 0.286, 1.0], // Discord input background
            border_color: [0.125, 0.129, 0.145, 1.0],
            text_color: [0.863, 0.867, 0.871, 1.0],
            border_width: 1.0,
            border_radius: 8.0, // Rounded for mobile
            opacity: 1.0,
            font_size: 16.0, // Prevent zoom on iOS
            font_family: "system-ui, sans-serif".to_string(),
            font_weight: FontWeight::Normal,
            text_align: TextAlign::Left,
            visible: true,
            z_index: 20,
        };
        
        ui.set_element_style(input_bar, style).await
            .map_err(|e| LogicError::SystemError(format!("Failed to style input bar: {}", e)))?;
        
        // Set bounds for mobile - full width at bottom with safe area
        ui.set_element_bounds(input_bar, ElementBounds::new(
            0.0,
            1080.0 - 80.0, // Position at bottom with space for safe area
            360.0, // Mobile width
            60.0, // Touch-friendly height
        )).await
            .map_err(|e| LogicError::SystemError(format!("Failed to set input bar bounds: {}", e)))?;
        
        Ok(input_bar)
    }
    
    /// Create a mobile Discord layout optimized for phones
    pub async fn create_mobile_discord_layout(&mut self) -> LogicResult<DiscordLayout> {
        // Simple logging using tracing for now since we can't access dashboard easily here
        tracing::info!("[UI-IF] create_mobile_discord_layout() called");
        
        let mut ui = self.ui_system.write().await;
        tracing::info!("[UI-IF] Got write lock on UI system");
        
        // Get root element
        let root_option = ui.get_root_element();
        tracing::info!("[UI-IF] get_root_element() returned: {:?}", root_option);
        
        let root = root_option
            .ok_or_else(|| {
                // Log more information about the state
                let initialized = ui.is_initialized();
                let error_msg = format!(
                    "No root element found. UI System initialized: {}. \
                    Make sure SystemsManager.initialize_all() was called before creating UI elements.",
                    initialized
                );
                tracing::error!("[UI-IF] ✗ {}", error_msg);
                LogicError::SystemError(error_msg)
            })?;
        
        tracing::info!("[UI-IF] ✓ Got root element: {:?}", root);
        
        // Mobile uses tabs or drawer for channel list, not side-by-side
        // Create main container with swipe navigation
        tracing::info!("[UI-IF] Creating main container...");
        let main_container = ui.create_element_with_id(
            "main-container".to_string(),
            "panel".to_string(),
            Some(root),
        ).await
            .map_err(|e| {
                tracing::error!("[UI-IF] Failed to create main container: {}", e);
                LogicError::SystemError(format!("Failed to create main container: {}", e))
            })?;
        tracing::info!("[UI-IF] Main container created: {:?}", main_container);
        
        // Full screen container
        ui.set_element_bounds(main_container, ElementBounds::new(0.0, 0.0, 360.0, 800.0))
            .await
            .map_err(|e| LogicError::SystemError(format!("Failed to set main container bounds: {}", e)))?;
        
        // Hidden drawer for channel list (swipe from left to show)
        let sidebar = ui.create_element_with_id(
            "channel-drawer".to_string(),
            "panel".to_string(),
            Some(main_container),
        ).await
            .map_err(|e| LogicError::SystemError(format!("Failed to create channel drawer: {}", e)))?;
        
        // Position off-screen initially
        ui.set_element_bounds(sidebar, ElementBounds::new(-280.0, 0.0, 280.0, 800.0))
            .await
            .map_err(|e| LogicError::SystemError(format!("Failed to set drawer bounds: {}", e)))?;
        
        // Main content area (messages)
        let main_content = ui.create_element_with_id(
            "main-content".to_string(),
            "panel".to_string(),
            Some(main_container),
        ).await
            .map_err(|e| LogicError::SystemError(format!("Failed to create main content: {}", e)))?;
        
        ui.set_element_bounds(main_content, ElementBounds::new(0.0, 0.0, 360.0, 800.0))
            .await
            .map_err(|e| LogicError::SystemError(format!("Failed to set main content bounds: {}", e)))?;
        
        // Message area with virtual keyboard consideration
        let message_area = ui.create_element_with_id(
            "message-area".to_string(),
            "scrollview".to_string(),
            Some(main_content),
        ).await
            .map_err(|e| LogicError::SystemError(format!("Failed to create message area: {}", e)))?;
        
        ui.set_element_bounds(message_area, ElementBounds::new(0.0, 50.0, 360.0, 690.0))
            .await
            .map_err(|e| LogicError::SystemError(format!("Failed to set message area bounds: {}", e)))?;
        
        // Input area at bottom
        let input_area = ui.create_element_with_id(
            "input-area".to_string(),
            "panel".to_string(),
            Some(main_content),
        ).await
            .map_err(|e| LogicError::SystemError(format!("Failed to create input area: {}", e)))?;
        
        ui.set_element_bounds(input_area, ElementBounds::new(0.0, 740.0, 360.0, 60.0))
            .await
            .map_err(|e| LogicError::SystemError(format!("Failed to set input area bounds: {}", e)))?;
        
        // Apply Discord mobile styles
        let sidebar_style = ElementStyle {
            background_color: [0.118, 0.122, 0.137, 0.95], // Darker with transparency
            ..Default::default()
        };
        ui.set_element_style(sidebar, sidebar_style).await.ok();
        
        let main_style = ElementStyle {
            background_color: [0.212, 0.224, 0.247, 1.0],
            ..Default::default()
        };
        ui.set_element_style(main_content, main_style).await.ok();
        
        Ok(DiscordLayout {
            sidebar,
            main_content,
            message_area,
            input_area,
            member_list: None, // No member list on mobile (accessed via menu)
        })
    }
    
    /// Add a message to the message area
    pub async fn add_message(
        &mut self,
        message_area: ElementId,
        username: &str,
        content: &str,
        timestamp: &str,
    ) -> LogicResult<ElementId> {
        let mut ui = self.ui_system.write().await;
        
        // Create message container
        let message = ui.create_element("panel", Some(message_area))
            .await
            .map_err(|e| LogicError::SystemError(format!("Failed to create message: {}", e)))?;
        
        // Create username text
        let username_elem = ui.create_element("text", Some(message))
            .await
            .map_err(|e| LogicError::SystemError(format!("Failed to create username: {}", e)))?;
        
        ui.set_element_text(username_elem, username.to_string())
            .await
            .map_err(|e| LogicError::SystemError(format!("Failed to set username: {}", e)))?;
        
        // Style username with Discord colors
        let username_style = ElementStyle {
            text_color: [0.4, 0.6, 1.0, 1.0], // Blue-ish for usernames
            font_size: 14.0,
            font_weight: FontWeight::Bold,
            ..Default::default()
        };
        ui.set_element_style(username_elem, username_style).await.ok();
        
        // Create message content
        let content_elem = ui.create_element("text", Some(message))
            .await
            .map_err(|e| LogicError::SystemError(format!("Failed to create content: {}", e)))?;
        
        ui.set_element_text(content_elem, content.to_string())
            .await
            .map_err(|e| LogicError::SystemError(format!("Failed to set content: {}", e)))?;
        
        Ok(message)
    }
}