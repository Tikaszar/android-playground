//! UI API module for systems/logic
//! 
//! This provides the public API for UI operations.
//! All functions forward to the UI command processor in systems/ui.

use playground_core_ui::{
    ElementId, ElementType, Style, Bounds, UiCommand,
    LayoutType, FlexLayout, GridLayout,
    ImageSource, KeyboardType, HapticType,
};
use playground_core_ecs::{EcsResult, SystemResponse};
use bytes::Bytes;

/// Create a new UI element
pub async fn create_element(
    element_type: ElementType,
    parent: Option<ElementId>
) -> EcsResult<ElementId> {
    let command = UiCommand::CreateElement {
        id: ElementId::new(),
        element_type,
        parent,
    };
    
    send_ui_command(command).await?;
    Ok(ElementId::new()) // Would get actual ID from response
}

/// Update an element's style
pub async fn update_element_style(id: ElementId, style: Style) -> EcsResult<()> {
    let command = UiCommand::UpdateStyle { id, style };
    send_ui_command(command).await?;
    Ok(())
}

/// Update an element's bounds
pub async fn update_element_bounds(id: ElementId, bounds: Bounds) -> EcsResult<()> {
    let command = UiCommand::UpdateBounds { id, bounds };
    send_ui_command(command).await?;
    Ok(())
}

/// Update an element's layout
pub async fn update_element_layout(
    id: ElementId,
    layout_type: LayoutType,
    flex: Option<FlexLayout>,
    grid: Option<GridLayout>,
) -> EcsResult<()> {
    let command = UiCommand::UpdateLayout {
        id,
        layout_type,
        flex,
        grid,
    };
    send_ui_command(command).await?;
    Ok(())
}

/// Set text content for an element
pub async fn set_element_text(id: ElementId, text: String) -> EcsResult<()> {
    let command = UiCommand::SetText { id, text };
    send_ui_command(command).await?;
    Ok(())
}

/// Set image source for an element
pub async fn set_element_image(id: ElementId, source: ImageSource) -> EcsResult<()> {
    let command = UiCommand::SetImage { id, source };
    send_ui_command(command).await?;
    Ok(())
}

/// Add a child element
pub async fn add_child_element(
    parent: ElementId,
    child: ElementId,
    index: Option<usize>
) -> EcsResult<()> {
    let command = UiCommand::AddChild { parent, child, index };
    send_ui_command(command).await?;
    Ok(())
}

/// Remove a child element
pub async fn remove_child_element(parent: ElementId, child: ElementId) -> EcsResult<()> {
    let command = UiCommand::RemoveChild { parent, child };
    send_ui_command(command).await?;
    Ok(())
}

/// Remove an element and all its children
pub async fn remove_element(id: ElementId) -> EcsResult<()> {
    let command = UiCommand::RemoveElement { id };
    send_ui_command(command).await?;
    Ok(())
}

/// Set element visibility
pub async fn set_element_visible(id: ElementId, visible: bool) -> EcsResult<()> {
    let command = UiCommand::SetVisible { id, visible };
    send_ui_command(command).await?;
    Ok(())
}

/// Set element enabled state
pub async fn set_element_enabled(id: ElementId, enabled: bool) -> EcsResult<()> {
    let command = UiCommand::SetEnabled { id, enabled };
    send_ui_command(command).await?;
    Ok(())
}

/// Focus an element
pub async fn focus_element(id: ElementId) -> EcsResult<()> {
    let command = UiCommand::Focus { id };
    send_ui_command(command).await?;
    Ok(())
}

/// Blur (unfocus) an element
pub async fn blur_element(id: ElementId) -> EcsResult<()> {
    let command = UiCommand::Blur { id };
    send_ui_command(command).await?;
    Ok(())
}

/// Scroll an element
pub async fn scroll_element(
    id: ElementId,
    x: Option<f32>,
    y: Option<f32>,
    animated: bool
) -> EcsResult<()> {
    let command = UiCommand::ScrollTo { id, x, y, animated };
    send_ui_command(command).await?;
    Ok(())
}

/// Show virtual keyboard (mobile)
pub async fn show_keyboard(input_type: KeyboardType) -> EcsResult<()> {
    let command = UiCommand::ShowKeyboard { input_type };
    send_ui_command(command).await?;
    Ok(())
}

/// Hide virtual keyboard (mobile)
pub async fn hide_keyboard() -> EcsResult<()> {
    let command = UiCommand::HideKeyboard;
    send_ui_command(command).await?;
    Ok(())
}

/// Set safe area insets (mobile)
pub async fn set_safe_area_insets(
    top: f32,
    bottom: f32,
    left: f32,
    right: f32
) -> EcsResult<()> {
    let command = UiCommand::SetSafeAreaInsets { top, bottom, left, right };
    send_ui_command(command).await?;
    Ok(())
}

/// Trigger haptic feedback (mobile)
pub async fn haptic_feedback(feedback_type: HapticType) -> EcsResult<()> {
    let command = UiCommand::HapticFeedback { feedback_type };
    send_ui_command(command).await?;
    Ok(())
}

/// Get the root element ID
pub async fn get_root_element() -> EcsResult<Option<ElementId>> {
    let response = playground_core_ecs::system_command_access::send_to_system(
        "ui",
        "get_root",
        Bytes::new()
    ).await?;
    
    // Parse response
    if let Some(payload) = response.payload {
        let root: Option<ElementId> = serde_json::from_slice(&payload)
            .map_err(|e| playground_core_ecs::EcsError::Generic(e.to_string()))?;
        Ok(root)
    } else {
        Ok(None)
    }
}

/// Trigger layout calculation
pub async fn perform_layout() -> EcsResult<()> {
    playground_core_ecs::system_command_access::send_to_system(
        "ui",
        "layout",
        Bytes::new()
    ).await?;
    Ok(())
}

// Internal helper to send UI commands
async fn send_ui_command(command: UiCommand) -> EcsResult<SystemResponse> {
    let payload = serde_json::to_vec(&command)
        .map_err(|e| playground_core_ecs::EcsError::Generic(e.to_string()))?;
    
    playground_core_ecs::system_command_access::send_to_system(
        "ui",
        "ui_command",
        Bytes::from(payload)
    ).await
}