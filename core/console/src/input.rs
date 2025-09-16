//! Input-related types (feature-gated)

#[cfg(feature = "input")]
use serde::{Deserialize, Serialize};

#[cfg(feature = "input")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum InputEvent {
    Text(String),
    Key(KeyCode),
    Pointer(PointerEvent),
    Resize { width: u32, height: u32 },
}

#[cfg(feature = "input")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum KeyCode {
    Enter,
    Escape,
    Backspace,
    Tab,
    Up,
    Down,
    Left,
    Right,
    Home,
    End,
    PageUp,
    PageDown,
    Delete,
    Function(u8),
    Char(char),
    Control(char),
    Alt(char),
}

#[cfg(feature = "input")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PointerEvent {
    pub x: f32,
    pub y: f32,
    pub button: Option<PointerButton>,
    pub event_type: PointerEventType,
}

#[cfg(feature = "input")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PointerButton {
    Primary,
    Secondary,
    Middle,
    Other(u8),
}

#[cfg(feature = "input")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PointerEventType {
    Move,
    Down,
    Up,
    Click,
    DoubleClick,
    Scroll { delta_x: f32, delta_y: f32 },
}