//! Input state component

use crate::input::*;
use crate::types::*;
use playground_core_ecs::impl_component_data;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

/// Input state as an ECS component
#[cfg(feature = "input")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InputStateComponent {
    /// Currently pressed keys
    pub pressed_keys: HashSet<KeyCode>,
    /// Mouse position (if available)
    pub mouse_position: Option<(Float, Float)>,
    /// Mouse button states
    pub mouse_buttons: PointerButtons,
    /// Active touch points
    #[serde(skip)]  // Can't serialize Touch struct easily
    pub touches: Vec<Touch>,
    /// Connected gamepads
    #[serde(skip)]  // Can't serialize GamepadState easily
    pub gamepads: Vec<GamepadState>,
}

#[cfg(feature = "input")]
impl_component_data!(InputStateComponent);

#[cfg(feature = "input")]
impl InputStateComponent {
    pub fn new() -> Self {
        Self {
            pressed_keys: HashSet::new(),
            mouse_position: None,
            mouse_buttons: PointerButtons::default(),
            touches: Vec::new(),
            gamepads: Vec::new(),
        }
    }

    pub fn handle_keyboard(&mut self, event: &KeyboardEvent) {
        match event.state {
            KeyState::Pressed => {
                self.pressed_keys.insert(event.key);
            }
            KeyState::Released => {
                self.pressed_keys.remove(&event.key);
            }
        }
    }

    pub fn handle_pointer(&mut self, event: &PointerEvent) {
        self.mouse_position = Some((event.x, event.y));
        self.mouse_buttons = event.buttons;
    }

    pub fn handle_touch(&mut self, event: &TouchEvent) {
        match event.event_type {
            TouchEventType::Start | TouchEventType::Move => {
                // Update or add touch point
                if let Some(touch) = self.touches.iter_mut().find(|t| t.id == event.id) {
                    touch.x = event.x;
                    touch.y = event.y;
                    touch.pressure = event.pressure;
                } else {
                    self.touches.push(Touch {
                        id: event.id,
                        x: event.x,
                        y: event.y,
                        pressure: event.pressure,
                    });
                }
            }
            TouchEventType::End | TouchEventType::Cancel => {
                self.touches.retain(|t| t.id != event.id);
            }
        }
    }

    pub fn is_key_pressed(&self, key: KeyCode) -> bool {
        self.pressed_keys.contains(&key)
    }
}

#[cfg(feature = "input")]
impl Default for InputStateComponent {
    fn default() -> Self {
        Self::new()
    }
}