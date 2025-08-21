use playground_core_ecs::{World, EntityId};
use playground_core_types::Shared;
use crate::error::{UiError, UiResult};
use crate::element::{ElementGraph, ElementId};
use crate::components::*;
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum InputEvent {
    MouseMove { x: f32, y: f32 },
    MouseDown { x: f32, y: f32, button: MouseButton },
    MouseUp { x: f32, y: f32, button: MouseButton },
    MouseWheel { delta: f32 },
    KeyDown { key: Key, modifiers: Modifiers },
    KeyUp { key: Key, modifiers: Modifiers },
    TextInput { text: String },
    TouchStart { id: u32, x: f32, y: f32 },
    TouchMove { id: u32, x: f32, y: f32 },
    TouchEnd { id: u32, x: f32, y: f32 },
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum MouseButton {
    Left,
    Right,
    Middle,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum Key {
    // Letters
    A, B, C, D, E, F, G, H, I, J, K, L, M,
    N, O, P, Q, R, S, T, U, V, W, X, Y, Z,
    
    // Numbers
    Num0, Num1, Num2, Num3, Num4,
    Num5, Num6, Num7, Num8, Num9,
    
    // Special keys
    Enter, Escape, Backspace, Tab, Space,
    Up, Down, Left, Right,
    Insert, Delete, Home, End, PageUp, PageDown,
    
    // Function keys
    F1, F2, F3, F4, F5, F6, F7, F8, F9, F10, F11, F12,
    
    // Modifiers (as keys)
    Shift, Control, Alt, Meta,
}

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
pub struct Modifiers {
    pub shift: bool,
    pub control: bool,
    pub alt: bool,
    pub meta: bool,
}

pub struct InputManager {
    hovered_element: Option<ElementId>,
    focused_element: Option<ElementId>,
    pressed_element: Option<ElementId>,
    mouse_position: [f32; 2],
    touches: std::collections::HashMap<u32, [f32; 2]>,
}

impl InputManager {
    pub fn new() -> Self {
        Self {
            hovered_element: None,
            focused_element: None,
            pressed_element: None,
            mouse_position: [0.0, 0.0],
            touches: std::collections::HashMap::new(),
        }
    }
    
    pub async fn process_event(
        &mut self,
        event: InputEvent,
        graph: &Shared<ElementGraph>,
        world: &Shared<World>,
    ) -> UiResult<bool> {
        match event {
            InputEvent::MouseMove { x, y } => {
                self.mouse_position = [x, y];
                self.update_hover(x, y, graph, world).await
            }
            InputEvent::MouseDown { x, y, button } => {
                if button == MouseButton::Left {
                    self.handle_mouse_down(x, y, graph, world).await
                } else {
                    Ok(false)
                }
            }
            InputEvent::MouseUp { x, y, button } => {
                if button == MouseButton::Left {
                    self.handle_mouse_up(x, y, graph, world).await
                } else {
                    Ok(false)
                }
            }
            InputEvent::KeyDown { key, modifiers } => {
                self.handle_key_down(key, modifiers, world).await
            }
            InputEvent::TextInput { text } => {
                self.handle_text_input(text, world).await
            }
            InputEvent::TouchStart { id, x, y } => {
                self.touches.insert(id, [x, y]);
                self.handle_mouse_down(x, y, graph, world).await
            }
            InputEvent::TouchMove { id, x, y } => {
                self.touches.insert(id, [x, y]);
                self.update_hover(x, y, graph, world).await
            }
            InputEvent::TouchEnd { id, x, y } => {
                self.touches.remove(&id);
                self.handle_mouse_up(x, y, graph, world).await
            }
            _ => Ok(false),
        }
    }
    
    pub fn get_focused_element(&self) -> Option<ElementId> {
        self.focused_element
    }
    
    async fn update_hover(
        &mut self,
        x: f32,
        y: f32,
        graph: &Shared<ElementGraph>,
        world: &Shared<World>,
    ) -> UiResult<bool> {
        let hit = self.hit_test(x, y, graph, world).await?;
        
        if hit != self.hovered_element {
            // Clear old hover
            if let Some(old) = self.hovered_element {
                let mut world_lock = world.write().await;
                if let Ok(mut elem) = world_lock.get_component_mut::<UiElementComponent>(old).await {
                    elem.hovered = false;
                }
            }
            
            // Set new hover
            if let Some(new) = hit {
                let mut world_lock = world.write().await;
                if let Ok(mut elem) = world_lock.get_component_mut::<UiElementComponent>(new).await {
                    elem.hovered = true;
                }
            }
            
            self.hovered_element = hit;
            Ok(true)
        } else {
            Ok(false)
        }
    }
    
    async fn handle_mouse_down(
        &mut self,
        x: f32,
        y: f32,
        graph: &Shared<ElementGraph>,
        world: &Shared<World>,
    ) -> UiResult<bool> {
        let hit = self.hit_test(x, y, graph, world).await?;
        
        if let Some(element) = hit {
            self.pressed_element = Some(element);
            
            // Update focus
            if self.focused_element != Some(element) {
                // Clear old focus
                if let Some(old) = self.focused_element {
                    let mut world_lock = world.write().await;
                    if let Ok(mut elem) = world_lock.get_component_mut::<UiElementComponent>(old).await {
                        elem.focused = false;
                    }
                }
                
                // Set new focus
                let mut world_lock = world.write().await;
                if let Ok(mut elem) = world_lock.get_component_mut::<UiElementComponent>(element).await {
                    elem.focused = true;
                }
                
                self.focused_element = Some(element);
            }
            
            Ok(true)
        } else {
            Ok(false)
        }
    }
    
    async fn handle_mouse_up(
        &mut self,
        x: f32,
        y: f32,
        graph: &Shared<ElementGraph>,
        world: &Shared<World>,
    ) -> UiResult<bool> {
        if let Some(pressed) = self.pressed_element {
            let hit = self.hit_test(x, y, graph, world).await?;
            
            // Click event if released on same element
            if hit == Some(pressed) {
                // Handle click
                let world_lock = world.read().await;
                let input = world_lock.get_component::<UiInputComponent>(pressed).await
                    .map_err(|e| UiError::EcsError(e.to_string()))?;
                
                if input.on_click.is_some() {
                    // Would trigger click handler here
                }
            }
            
            self.pressed_element = None;
            Ok(true)
        } else {
            Ok(false)
        }
    }
    
    async fn handle_key_down(
        &mut self,
        key: Key,
        _modifiers: Modifiers,
        world: &Shared<World>,
    ) -> UiResult<bool> {
        if let Some(focused) = self.focused_element {
            let world_lock = world.read().await;
            
            // Check if element accepts input
            let input = world_lock.get_component::<UiInputComponent>(focused).await
                .map_err(|e| UiError::EcsError(e.to_string()))?;
            
            if input.accepts_input {
                // Handle special keys
                match key {
                    Key::Tab => {
                        // Tab navigation would go here
                    }
                    Key::Escape => {
                        // Clear focus
                        drop(world_lock);
                        let mut world_lock = world.write().await;
                        if let Ok(mut elem) = world_lock.get_component_mut::<UiElementComponent>(focused).await {
                            elem.focused = false;
                        }
                        self.focused_element = None;
                    }
                    _ => {}
                }
                
                Ok(true)
            } else {
                Ok(false)
            }
        } else {
            Ok(false)
        }
    }
    
    async fn handle_text_input(
        &mut self,
        text: String,
        world: &Shared<World>,
    ) -> UiResult<bool> {
        if let Some(focused) = self.focused_element {
            let mut world_lock = world.write().await;
            
            // Check if element has text component
            if let Ok(mut text_comp) = world_lock.get_component_mut::<UiTextComponent>(focused).await {
                if text_comp.editable {
                    // Insert text at cursor position
                    text_comp.text.insert_str(text_comp.cursor_position, &text);
                    text_comp.cursor_position += text.len();
                    
                    // Update element text content
                    if let Ok(mut elem) = world_lock.get_component_mut::<UiElementComponent>(focused).await {
                        elem.text_content = Some(text_comp.text.clone());
                    }
                    
                    return Ok(true);
                }
            }
        }
        
        Ok(false)
    }
    
    async fn hit_test(
        &self,
        x: f32,
        y: f32,
        graph: &Shared<ElementGraph>,
        world: &Shared<World>,
    ) -> UiResult<Option<ElementId>> {
        // Simple hit test - find topmost element containing point
        // In real implementation, would traverse tree in reverse order
        
        let world_lock = world.read().await;
        let graph_lock = graph.read().await;
        
        // For now, just check if we have any elements
        // Real implementation would check bounds
        
        Ok(None)
    }
}