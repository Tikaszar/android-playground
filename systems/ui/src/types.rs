use serde::{Serialize, Deserialize};
use crate::element::ElementId;

/// Style properties for UI elements - simple data structure for plugins
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ElementStyle {
    pub background_color: [f32; 4],
    pub text_color: [f32; 4],
    pub border_color: [f32; 4],
    pub border_width: f32,
    pub border_radius: f32,
    pub opacity: f32,
    pub font_size: f32,
    pub font_family: String,
    pub font_weight: FontWeight,
    pub text_align: TextAlign,
    pub visible: bool,
    pub z_index: i32,
}

impl Default for ElementStyle {
    fn default() -> Self {
        Self {
            background_color: [1.0, 1.0, 1.0, 1.0],
            text_color: [0.0, 0.0, 0.0, 1.0],
            border_color: [0.0, 0.0, 0.0, 1.0],
            border_width: 0.0,
            border_radius: 0.0,
            opacity: 1.0,
            font_size: 14.0,
            font_family: "sans-serif".to_string(),
            font_weight: FontWeight::Normal,
            text_align: TextAlign::Left,
            visible: true,
            z_index: 0,
        }
    }
}

/// Font weight options
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum FontWeight {
    Light,
    Normal,
    Bold,
    ExtraBold,
}

/// Text alignment options
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum TextAlign {
    Left,
    Center,
    Right,
    Justify,
}

/// Bounds for UI elements
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct ElementBounds {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

impl ElementBounds {
    pub fn new(x: f32, y: f32, width: f32, height: f32) -> Self {
        Self { x, y, width, height }
    }
    
    pub fn contains(&self, x: f32, y: f32) -> bool {
        x >= self.x && x <= self.x + self.width &&
        y >= self.y && y <= self.y + self.height
    }
}

/// Layout type for elements
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum LayoutType {
    Absolute,
    Flexbox,
    Docking,
}

/// Flexbox properties
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FlexboxLayout {
    pub direction: FlexDirection,
    pub justify_content: JustifyContent,
    pub align_items: AlignItems,
    pub gap: f32,
    pub padding: [f32; 4], // top, right, bottom, left
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum FlexDirection {
    Row,
    Column,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum JustifyContent {
    Start,
    Center,
    End,
    SpaceBetween,
    SpaceAround,
    SpaceEvenly,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum AlignItems {
    Start,
    Center,
    End,
    Stretch,
}

/// High-level Discord-style layout structure
#[derive(Debug, Clone)]
pub struct DiscordLayout {
    pub sidebar: ElementId,
    pub main_content: ElementId,
    pub message_area: ElementId,
    pub input_area: ElementId,
    pub member_list: Option<ElementId>,
}

/// UI event types that plugins can handle
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum UiEvent {
    Click { element: ElementId, x: f32, y: f32 },
    TouchStart { element: ElementId, x: f32, y: f32 },
    TouchMove { element: ElementId, x: f32, y: f32 },
    TouchEnd { element: ElementId, x: f32, y: f32 },
    KeyDown { key: String, shift: bool, ctrl: bool, alt: bool },
    KeyUp { key: String, shift: bool, ctrl: bool, alt: bool },
    TextInput { text: String },
    Resize { width: f32, height: f32 },
    Focus { element: ElementId },
    Blur { element: ElementId },
}