//! UI system command processor for handling UI commands through ECS

use async_trait::async_trait;
use playground_core_ecs::{
    SystemCommandProcessor, SystemResponse, EcsResult, EcsError
};
use playground_core_ui::{
    UiCommand, ElementUpdate, ImageSource, KeyboardType, HapticType
};
use crate::system::UiSystem;
use crate::types::{ElementStyle, ElementBounds};
use bytes::Bytes;
use serde_json;
use std::sync::Arc;
use tokio::sync::RwLock;

/// UI command processor implementation
pub struct UiCommandProcessor {
    ui_system: Arc<RwLock<UiSystem>>,
}

impl UiCommandProcessor {
    pub fn new(ui_system: Arc<RwLock<UiSystem>>) -> Self {
        Self { ui_system }
    }
}

#[async_trait]
impl SystemCommandProcessor for UiCommandProcessor {
    fn system_name(&self) -> &str {
        "ui"
    }
    
    async fn handle_system_command(&self, command_type: &str, payload: Bytes) -> EcsResult<SystemResponse> {
        match command_type {
            "ui_command" => {
                // Deserialize UI command from payload
                let command: UiCommand = serde_json::from_slice(&payload)
                    .map_err(|e| EcsError::Generic(format!("Failed to deserialize UI command: {}", e)))?;
                
                // Process the command
                let result = self.process_ui_command(command).await;
                
                match result {
                    Ok(response_data) => {
                        let response_bytes = serde_json::to_vec(&response_data)
                            .map_err(|e| EcsError::Generic(e.to_string()))?;
                        
                        Ok(SystemResponse {
                            success: true,
                            payload: Some(Bytes::from(response_bytes)),
                            error: None,
                        })
                    },
                    Err(e) => {
                        Ok(SystemResponse {
                            success: false,
                            payload: None,
                            error: Some(e.to_string()),
                        })
                    }
                }
            },
            "get_element" => {
                // Get element data
                let element_id: playground_core_ui::ElementId = serde_json::from_slice(&payload)
                    .map_err(|e| EcsError::Generic(format!("Failed to deserialize element ID: {}", e)))?;
                
                let ui = self.ui_system.read().await;
                let element_data = ui.storage.get_element(element_id).await
                    .ok_or_else(|| EcsError::Generic(format!("Element not found")))?
                
                let response_bytes = serde_json::to_vec(&element_data)
                    .map_err(|e| EcsError::Generic(e.to_string()))?;
                
                Ok(SystemResponse {
                    success: true,
                    payload: Some(Bytes::from(response_bytes)),
                    error: None,
                })
            },
            "get_root" => {
                // Get root element ID
                let ui = self.ui_system.read().await;
                let root = ui.get_root_element().await;
                
                let response_bytes = serde_json::to_vec(&root)
                    .map_err(|e| EcsError::Generic(e.to_string()))?;
                
                Ok(SystemResponse {
                    success: true,
                    payload: Some(Bytes::from(response_bytes)),
                    error: None,
                })
            },
            "layout" => {
                // Trigger layout calculation
                let mut ui = self.ui_system.write().await;
                ui.perform_layout().await
                    .map_err(|e| EcsError::Generic(e.to_string()))?;
                
                Ok(SystemResponse {
                    success: true,
                    payload: None,
                    error: None,
                })
            },
            _ => {
                Err(EcsError::Generic(format!("Unknown UI command type: {}", command_type)))
            }
        }
    }
    
    fn supported_commands(&self) -> Vec<String> {
        vec![
            "ui_command".to_string(),
            "get_element".to_string(),
            "get_root".to_string(),
            "layout".to_string(),
        ]
    }
}

impl UiCommandProcessor {
    async fn process_ui_command(&self, command: UiCommand) -> Result<serde_json::Value, EcsError> {
        let mut ui = self.ui_system.write().await;
        
        match command {
            UiCommand::CreateElement { id, element_type, parent } => {
                // Create element returns a new ID, but we want to use the provided ID
                // For now, create and then update if needed
                let new_id = ui.storage.create_element(element_type).await;
                
                // If parent is specified, add as child
                if let Some(parent_id) = parent {
                    ui.element_graph.write().await.add_child(&parent_id, &new_id, None)
                        .map_err(|e| EcsError::Generic(e.to_string()))?;
                }
                
                Ok(serde_json::json!({ "created": new_id }))
            },
            UiCommand::UpdateStyle { id, style } => {
                // Convert from core/ui Style to systems/ui ElementStyle
                let element_style = ElementStyle {
                    background_color: style.background_color.map(|c| [c.r, c.g, c.b, c.a]),
                    text_color: style.text_color.map(|c| [c.r, c.g, c.b, c.a]),
                    border_color: style.border_color.map(|c| [c.r, c.g, c.b, c.a]),
                    border_width: style.border_width.unwrap_or(0.0),
                    border_radius: style.border_radius.unwrap_or(0.0),
                    padding: [
                        style.padding_top.unwrap_or(0.0),
                        style.padding_right.unwrap_or(0.0),
                        style.padding_bottom.unwrap_or(0.0),
                        style.padding_left.unwrap_or(0.0),
                    ],
                    margin: [
                        style.margin_top.unwrap_or(0.0),
                        style.margin_right.unwrap_or(0.0),
                        style.margin_bottom.unwrap_or(0.0),
                        style.margin_left.unwrap_or(0.0),
                    ],
                    font_size: style.font_size,
                    font_family: style.font_family,
                    font_weight: crate::types::FontWeight::Normal,
                    text_align: crate::types::TextAlign::Left,
                    opacity: style.opacity.unwrap_or(1.0),
                    z_index: style.z_index.unwrap_or(0),
                };
                
                let updated = ui.storage.update_element(id.clone(), |elem| {
                    elem.style = element_style;
                }).await;
                
                if !updated {
                    return Err(EcsError::Generic(format!("Element {} not found", id)));
                }
                Ok(serde_json::json!({ "updated": id }))
            },
            UiCommand::UpdateBounds { id, bounds } => {
                let element_bounds = ElementBounds {
                    x: bounds.x,
                    y: bounds.y,
                    width: bounds.width,
                    height: bounds.height,
                };
                
                let updated = ui.storage.update_element(id.clone(), |elem| {
                    elem.bounds = element_bounds;
                }).await;
                
                if !updated {
                    return Err(EcsError::Generic(format!("Element {} not found", id)));
                }
                Ok(serde_json::json!({ "updated": id }))
            },
            UiCommand::SetText { id, text } => {
                let updated = ui.storage.update_element(id.clone(), |elem| {
                    elem.text = Some(text);
                }).await;
                
                if !updated {
                    return Err(EcsError::Generic(format!("Element {} not found", id)));
                }
                Ok(serde_json::json!({ "updated": id }))
            },
            UiCommand::SetVisible { id, visible } => {
                let updated = ui.storage.update_element(id.clone(), |elem| {
                    elem.visible = visible;
                }).await;
                
                if !updated {
                    return Err(EcsError::Generic(format!("Element {} not found", id)));
                }
                Ok(serde_json::json!({ "updated": id }))
            },
            UiCommand::SetEnabled { id, enabled } => {
                let updated = ui.storage.update_element(id.clone(), |elem| {
                    elem.enabled = enabled;
                }).await;
                
                if !updated {
                    return Err(EcsError::Generic(format!("Element {} not found", id)));
                }
                Ok(serde_json::json!({ "updated": id }))
            },
            UiCommand::AddChild { parent, child, index } => {
                ui.element_graph.write().await.add_child(&parent, &child, index)
                    .map_err(|e| EcsError::Generic(e.to_string()))?;
                Ok(serde_json::json!({ "parent": parent, "child": child }))
            },
            UiCommand::RemoveChild { parent, child } => {
                ui.element_graph.write().await.remove_child(&parent, &child)
                    .map_err(|e| EcsError::Generic(e.to_string()))?;
                Ok(serde_json::json!({ "parent": parent, "child": child }))
            },
            UiCommand::RemoveElement { id } => {
                let removed = ui.storage.remove_element(id.clone()).await;
                if !removed {
                    return Err(EcsError::Generic(format!("Element {} not found", id)));
                }
                Ok(serde_json::json!({ "removed": id }))
            },
            UiCommand::Focus { id } => {
                ui.input_manager.write().await.set_focused_element(Some(id.clone()));
                Ok(serde_json::json!({ "focused": id }))
            },
            UiCommand::Blur { id } => {
                ui.input_manager.write().await.set_focused_element(None);
                Ok(serde_json::json!({ "blurred": id }))
            },
            _ => {
                Err(EcsError::Generic("UI command not yet implemented".to_string()))
            }
        }
    }
}