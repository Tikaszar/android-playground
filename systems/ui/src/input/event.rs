//! Input event types

use nalgebra::Vector2;
use serde::{Deserialize, Serialize};

/// Result of handling an input event
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EventHandled {
    Yes,
    No,
}

/// Result of input handling
#[derive(Debug, Clone, Copy)]
pub struct InputResult {
    pub handled: EventHandled,
    pub request_focus: bool,
}

impl Default for InputResult {
    fn default() -> Self {
        Self {
            handled: EventHandled::No,
            request_focus: false,
        }
    }
}

/// Input event types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum InputEvent {
    /// Mouse/touch moved
    PointerMove {
        position: Vector2<f32>,
        delta: Vector2<f32>,
    },
    /// Mouse/touch down
    PointerDown {
        position: Vector2<f32>,
        button: PointerButton,
    },
    /// Mouse/touch up
    PointerUp {
        position: Vector2<f32>,
        button: PointerButton,
    },
    /// Scroll/pinch
    Scroll {
        position: Vector2<f32>,
        delta: Vector2<f32>,
    },
    /// Pinch gesture
    Pinch {
        center: Vector2<f32>,
        scale: f32,
    },
    /// Key down
    KeyDown {
        key: Key,
        modifiers: Modifiers,
    },
    /// Key up
    KeyUp {
        key: Key,
        modifiers: Modifiers,
    },
    /// Text input
    TextInput {
        text: String,
    },
    /// Focus changed
    FocusChanged {
        focused: bool,
    },
}

/// Pointer button
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PointerButton {
    Primary,
    Secondary,
    Middle,
    Touch(u32),
}

/// Keyboard key
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Key {
    // Letters
    A, B, C, D, E, F, G, H, I, J, K, L, M,
    N, O, P, Q, R, S, T, U, V, W, X, Y, Z,
    
    // Numbers
    Num0, Num1, Num2, Num3, Num4,
    Num5, Num6, Num7, Num8, Num9,
    
    // Function keys
    F1, F2, F3, F4, F5, F6, F7, F8,
    F9, F10, F11, F12,
    
    // Navigation
    Up, Down, Left, Right,
    Home, End, PageUp, PageDown,
    
    // Editing
    Backspace, Delete, Tab, Enter,
    Space, Escape,
    
    // Modifiers
    Shift, Control, Alt, Meta,
    
    // Other
    Unknown,
}

/// Keyboard modifiers
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Modifiers {
    pub shift: bool,
    pub control: bool,
    pub alt: bool,
    pub meta: bool,
}

impl Default for Modifiers {
    fn default() -> Self {
        Self {
            shift: false,
            control: false,
            alt: false,
            meta: false,
        }
    }
}