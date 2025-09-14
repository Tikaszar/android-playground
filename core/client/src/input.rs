//! Generic input contracts for clients

use serde::{Deserialize, Serialize};

/// Generic input event that any client can produce
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum InputEvent {
    /// Keyboard input
    Keyboard(KeyboardEvent),
    /// Mouse/pointer input
    Pointer(PointerEvent),
    /// Touch input (mobile/tablet)
    Touch(TouchEvent),
    /// Gamepad/controller input
    Gamepad(GamepadEvent),
    /// Text input (for IME/composition)
    Text(TextEvent),
    /// Window/surface events
    Window(WindowEvent),
    /// Custom input event
    Custom(String, serde_json::Value),
}

/// Keyboard event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyboardEvent {
    /// Key that was pressed/released
    pub key: KeyCode,
    /// Was the key pressed or released
    pub state: KeyState,
    /// Modifier keys held
    pub modifiers: Modifiers,
    /// Repeat event
    pub repeat: bool,
}

/// Key state
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum KeyState {
    Pressed,
    Released,
}

/// Generic key codes (not tied to any specific platform)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum KeyCode {
    // Letters
    A, B, C, D, E, F, G, H, I, J, K, L, M,
    N, O, P, Q, R, S, T, U, V, W, X, Y, Z,
    
    // Numbers
    Num0, Num1, Num2, Num3, Num4,
    Num5, Num6, Num7, Num8, Num9,
    
    // Function keys
    F1, F2, F3, F4, F5, F6, F7, F8,
    F9, F10, F11, F12,
    
    // Control keys
    Escape,
    Enter,
    Tab,
    Backspace,
    Space,
    Delete,
    Insert,
    Home,
    End,
    PageUp,
    PageDown,
    
    // Arrow keys
    Up, Down, Left, Right,
    
    // Modifiers
    LeftShift, RightShift,
    LeftControl, RightControl,
    LeftAlt, RightAlt,
    LeftSuper, RightSuper, // Windows/Command key
    
    // Other
    CapsLock,
    NumLock,
    ScrollLock,
    Pause,
    PrintScreen,
    
    // Unknown key
    Unknown(u32),
}

/// Modifier key flags
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
pub struct Modifiers {
    pub shift: bool,
    pub control: bool,
    pub alt: bool,
    pub super_key: bool, // Windows/Command
}

/// Pointer/mouse event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PointerEvent {
    /// Pointer ID (for multi-pointer support)
    pub id: u32,
    /// Position in client coordinates
    pub x: f32,
    pub y: f32,
    /// Button states
    pub buttons: PointerButtons,
    /// Event type
    pub event_type: PointerEventType,
    /// Scroll delta (if scrolling)
    pub scroll_delta: Option<(f32, f32)>,
    /// Pressure (for stylus/touch)
    pub pressure: Option<f32>,
}

/// Pointer button states
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
pub struct PointerButtons {
    pub primary: bool,   // Usually left
    pub secondary: bool, // Usually right
    pub middle: bool,
    pub back: bool,
    pub forward: bool,
}

/// Pointer event type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PointerEventType {
    Move,
    Down,
    Up,
    Enter,
    Leave,
    Scroll,
}

/// Touch event (for multi-touch)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TouchEvent {
    /// Touch point ID
    pub id: u32,
    /// Position in client coordinates
    pub x: f32,
    pub y: f32,
    /// Touch radius
    pub radius_x: f32,
    pub radius_y: f32,
    /// Rotation angle
    pub rotation: f32,
    /// Pressure
    pub pressure: f32,
    /// Event type
    pub event_type: TouchEventType,
}

/// Touch event type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TouchEventType {
    Start,
    Move,
    End,
    Cancel,
}

/// Gamepad event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GamepadEvent {
    /// Gamepad ID
    pub id: u32,
    /// Event type
    pub event_type: GamepadEventType,
}

/// Gamepad event type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GamepadEventType {
    Connected,
    Disconnected,
    ButtonPressed { button: u32, value: f32 },
    ButtonReleased { button: u32 },
    AxisMoved { axis: u32, value: f32 },
}

/// Text input event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextEvent {
    /// The text that was input
    pub text: String,
    /// Is this part of a composition (IME)
    pub is_composing: bool,
}

/// Window event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WindowEvent {
    /// Window resized
    Resized { width: u32, height: u32 },
    /// Window moved
    Moved { x: i32, y: i32 },
    /// Window gained/lost focus
    FocusChanged { focused: bool },
    /// Window minimized/restored
    Minimized(bool),
    /// Window maximized/restored
    Maximized(bool),
    /// Window close requested
    CloseRequested,
    /// DPI scale changed
    ScaleChanged { scale: f32 },
}