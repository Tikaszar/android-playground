use crate::types::ElementId;
use serde::{Serialize, Deserialize};

/// UI events that can be handled
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum UiEvent {
    /// Touch events (mobile-first)
    TouchStart {
        element: ElementId,
        x: f32,
        y: f32,
        touch_id: u32,
    },
    
    TouchMove {
        element: ElementId,
        x: f32,
        y: f32,
        touch_id: u32,
        delta_x: f32,
        delta_y: f32,
    },
    
    TouchEnd {
        element: ElementId,
        x: f32,
        y: f32,
        touch_id: u32,
    },
    
    TouchCancel {
        element: ElementId,
        touch_id: u32,
    },
    
    /// Gesture events (mobile-specific)
    Tap {
        element: ElementId,
        x: f32,
        y: f32,
    },
    
    DoubleTap {
        element: ElementId,
        x: f32,
        y: f32,
    },
    
    LongPress {
        element: ElementId,
        x: f32,
        y: f32,
        duration_ms: u64,
    },
    
    Swipe {
        element: ElementId,
        direction: SwipeDirection,
        velocity: f32,
        distance: f32,
    },
    
    Pinch {
        element: ElementId,
        scale: f32,
        center_x: f32,
        center_y: f32,
    },
    
    Rotate {
        element: ElementId,
        angle: f32,
        center_x: f32,
        center_y: f32,
    },
    
    /// Scroll events
    Scroll {
        element: ElementId,
        delta_x: f32,
        delta_y: f32,
        content_offset_x: f32,
        content_offset_y: f32,
    },
    
    ScrollStart {
        element: ElementId,
    },
    
    ScrollEnd {
        element: ElementId,
    },
    
    /// Text input events
    TextInput {
        element: ElementId,
        text: String,
    },
    
    TextChanged {
        element: ElementId,
        text: String,
    },
    
    /// Focus events
    Focus {
        element: ElementId,
    },
    
    Blur {
        element: ElementId,
    },
    
    /// Keyboard events (when virtual keyboard is shown)
    KeyDown {
        key: String,
        code: String,
        shift: bool,
        ctrl: bool,
        alt: bool,
        meta: bool,
    },
    
    KeyUp {
        key: String,
        code: String,
        shift: bool,
        ctrl: bool,
        alt: bool,
        meta: bool,
    },
    
    /// Layout events
    Resize {
        width: f32,
        height: f32,
        orientation: Orientation,
    },
    
    OrientationChange {
        orientation: Orientation,
    },
    
    /// Visibility events
    Show {
        element: ElementId,
    },
    
    Hide {
        element: ElementId,
    },
    
    /// Lifecycle events
    Mount {
        element: ElementId,
    },
    
    Unmount {
        element: ElementId,
    },
    
    /// Accessibility events
    AccessibilityAction {
        element: ElementId,
        action: AccessibilityAction,
    },
}

/// Swipe direction for gesture recognition
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum SwipeDirection {
    Up,
    Down,
    Left,
    Right,
}

/// Device orientation
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum Orientation {
    Portrait,
    PortraitUpsideDown,
    LandscapeLeft,
    LandscapeRight,
}

/// Accessibility actions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AccessibilityAction {
    Tap,
    DoubleTap,
    LongPress,
    Scroll(SwipeDirection),
    Focus,
    Escape,
    MagicTap,
    Custom(String),
}

/// Event propagation control
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum EventResult {
    /// Event was handled, stop propagation
    Handled,
    /// Event was not handled, continue propagation
    Ignored,
    /// Event was partially handled, continue with modified event
    Modified,
}