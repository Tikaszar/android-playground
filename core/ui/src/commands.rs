use crate::types::{ElementId, ElementType, Style, Bounds, LayoutType, FlexLayout, GridLayout};
use serde::{Serialize, Deserialize};

/// Commands for UI operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum UiCommand {
    /// Create a new UI element
    CreateElement {
        id: ElementId,
        element_type: ElementType,
        parent: Option<ElementId>,
    },
    
    /// Update an element's style
    UpdateStyle {
        id: ElementId,
        style: Style,
    },
    
    /// Update an element's bounds
    UpdateBounds {
        id: ElementId,
        bounds: Bounds,
    },
    
    /// Update an element's layout
    UpdateLayout {
        id: ElementId,
        layout_type: LayoutType,
        flex: Option<FlexLayout>,
        grid: Option<GridLayout>,
    },
    
    /// Set text content
    SetText {
        id: ElementId,
        text: String,
    },
    
    /// Set image source
    SetImage {
        id: ElementId,
        source: ImageSource,
    },
    
    /// Add a child element
    AddChild {
        parent: ElementId,
        child: ElementId,
        index: Option<usize>,
    },
    
    /// Remove a child element
    RemoveChild {
        parent: ElementId,
        child: ElementId,
    },
    
    /// Remove an element and all its children
    RemoveElement {
        id: ElementId,
    },
    
    /// Set element visibility
    SetVisible {
        id: ElementId,
        visible: bool,
    },
    
    /// Set element enabled state
    SetEnabled {
        id: ElementId,
        enabled: bool,
    },
    
    /// Focus an element
    Focus {
        id: ElementId,
    },
    
    /// Blur (unfocus) an element
    Blur {
        id: ElementId,
    },
    
    /// Scroll an element
    ScrollTo {
        id: ElementId,
        x: Option<f32>,
        y: Option<f32>,
        animated: bool,
    },
    
    /// Mobile-specific: Show virtual keyboard
    ShowKeyboard {
        input_type: KeyboardType,
    },
    
    /// Mobile-specific: Hide virtual keyboard
    HideKeyboard,
    
    /// Mobile-specific: Set safe area insets
    SetSafeAreaInsets {
        top: f32,
        bottom: f32,
        left: f32,
        right: f32,
    },
    
    /// Mobile-specific: Haptic feedback
    HapticFeedback {
        feedback_type: HapticType,
    },
}

/// Image source types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ImageSource {
    Url(String),
    Path(String),
    Bytes(Vec<u8>),
    ResourceId(String),
}

/// Mobile keyboard types
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum KeyboardType {
    Default,
    Numeric,
    Email,
    Phone,
    Url,
    Search,
}

/// Mobile haptic feedback types
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum HapticType {
    Light,
    Medium,
    Heavy,
    Selection,
    Success,
    Warning,
    Error,
}

/// Update types for element modifications
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ElementUpdate {
    Style(Style),
    Bounds(Bounds),
    Text(String),
    Image(ImageSource),
    Layout(LayoutType),
    Visible(bool),
    Enabled(bool),
    Children(Vec<ElementId>),
}