use playground_core_ecs::{World, EntityId};
use playground_core_types::{Shared, Handle};
use std::collections::HashMap;
use crate::error::{UiError, UiResult};
use crate::element::{ElementGraph, ElementId};
use crate::components::{UiElementComponent, UiInputComponent, UiTextComponent};
use super::event::{InputEvent, MouseButton, Key, Modifiers};

pub struct InputManager {
    hovered_element: Option<ElementId>,
    focused_element: Option<ElementId>,
    pressed_element: Option<ElementId>,
    mouse_position: [f32; 2],
    touches: HashMap<u32, [f32; 2]>,
}

impl InputManager {
    pub fn new() -> Self {
        Self {
            hovered_element: None,
            focused_element: None,
            pressed_element: None,
            mouse_position: [0.0, 0.0],
            touches: HashMap::new(),
        }
    }
    
    pub async fn process_event(
        &mut self,
        event: InputEvent,
        graph: &Shared<ElementGraph>,
        world: &Handle<World>,
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
        world: &Handle<World>,
    ) -> UiResult<bool> {
        let hit = self.hit_test(x, y, graph, world).await?;
        
        if hit != self.hovered_element {
            // Clear old hover
            if let Some(old) = self.hovered_element {
                let _ = world.update_component::<UiElementComponent>(old, |elem| {
                    elem.hovered = false;
                }).await;
            }
            
            // Set new hover
            if let Some(new) = hit {
                let _ = world.update_component::<UiElementComponent>(new, |elem| {
                    elem.hovered = true;
                }).await;
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
        world: &Handle<World>,
    ) -> UiResult<bool> {
        let hit = self.hit_test(x, y, graph, world).await?;
        
        if let Some(element) = hit {
            self.pressed_element = Some(element);
            
            // Update focus
            if self.focused_element != Some(element) {
                // Clear old focus
                if let Some(old) = self.focused_element {
                    let _ = world.update_component::<UiElementComponent>(old, |elem| {
                        elem.focused = false;
                    }).await;
                }
                
                // Set new focus
                let _ = world.update_component::<UiElementComponent>(element, |elem| {
                    elem.focused = true;
                }).await;
                
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
        world: &Handle<World>,
    ) -> UiResult<bool> {
        if let Some(pressed) = self.pressed_element {
            let hit = self.hit_test(x, y, graph, world).await?;
            
            // Click event if released on same element
            if hit == Some(pressed) {
                // Handle click - world is Handle<World> now, call methods directly
                let input = world.get_component::<UiInputComponent>(pressed).await
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
        world: &Handle<World>,
    ) -> UiResult<bool> {
        if let Some(focused) = self.focused_element {
            // Check if element accepts input - world is Handle<World> now
            let input = world.get_component::<UiInputComponent>(focused).await
                .map_err(|e| UiError::EcsError(e.to_string()))?;
            
            if input.accepts_input {
                // Handle special keys
                match key {
                    Key::Tab => {
                        // Tab navigation would go here
                    }
                    Key::Escape => {
                        // Clear focus - world is Handle<World> now
                        let _ = world.update_component::<UiElementComponent>(focused, |elem| {
                            elem.focused = false;
                        }).await;
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
        world: &Handle<World>,
    ) -> UiResult<bool> {
        if let Some(focused) = self.focused_element {
            // Check if element has text component - world is Handle<World> now
            if let Ok(text_comp) = world.get_component::<UiTextComponent>(focused).await {
                if text_comp.editable {
                    let new_text = {
                        let mut text_content = text_comp.text.clone();
                        text_content.insert_str(text_comp.cursor_position, &text);
                        text_content
                    };
                    let new_cursor = text_comp.cursor_position + text.len();
                    
                    // Update text component
                    world.update_component::<UiTextComponent>(focused, |tc| {
                        tc.text = new_text.clone();
                        tc.cursor_position = new_cursor;
                    }).await.map_err(|e| UiError::EcsError(e.to_string()))?;
                    
                    // Update element text content
                    world.update_component::<UiElementComponent>(focused, |elem| {
                        elem.text_content = Some(new_text);
                    }).await.map_err(|e| UiError::EcsError(e.to_string()))?;
                    
                    return Ok(true);
                }
            }
        }
        
        Ok(false)
    }
    
    async fn hit_test(
        &self,
        _x: f32,
        _y: f32,
        _graph: &Shared<ElementGraph>,
        _world: &Handle<World>,
    ) -> UiResult<Option<ElementId>> {
        // Simple hit test - find topmost element containing point
        // In real implementation, would traverse tree in reverse order
        
        // For now, just return None
        // Real implementation would check bounds
        
        Ok(None)
    }
}