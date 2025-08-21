use playground_core_ecs::{Component, ComponentId, EcsError};
use serde::{Serialize, Deserialize};
use nalgebra::Vector4;
use std::collections::HashMap;

// Element component
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiElementComponent {
    pub id: String,
    pub element_type: String,
    pub text_content: Option<String>,
    pub visible: bool,
    pub hovered: bool,
    pub focused: bool,
    pub disabled: bool,
    pub children: Vec<playground_core_ecs::EntityId>,
}

impl UiElementComponent {
    pub fn new(element_type: &str) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            element_type: element_type.to_string(),
            text_content: None,
            visible: true,
            hovered: false,
            focused: false,
            disabled: false,
            children: Vec::new(),
        }
    }
}

impl Component for UiElementComponent {
    fn component_id() -> ComponentId {
        ComponentId(100)
    }
    
    fn component_name() -> &'static str {
        "UiElementComponent"
    }
    
    fn serialize(&self) -> Result<Vec<u8>, EcsError> {
        bincode::serialize(self)
            .map_err(|e| EcsError::SerializationFailed(e.to_string()))
    }
    
    fn deserialize(data: &[u8]) -> Result<Self, EcsError> {
        bincode::deserialize(data)
            .map_err(|e| EcsError::DeserializationFailed(e.to_string()))
    }
}

// Layout component
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiLayoutComponent {
    pub bounds: ElementBounds,
    pub padding: [f32; 4], // top, right, bottom, left
    pub margin: [f32; 4],
    pub layout_type: LayoutType,
    pub flex_direction: FlexDirection,
    pub justify_content: JustifyContent,
    pub align_items: AlignItems,
    pub position_type: PositionType,
    pub size: Size,
    pub min_size: Size,
    pub max_size: Size,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct ElementBounds {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum LayoutType {
    Flexbox,
    Absolute,
    Docking,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum FlexDirection {
    Row,
    Column,
    RowReverse,
    ColumnReverse,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum JustifyContent {
    FlexStart,
    FlexEnd,
    Center,
    SpaceBetween,
    SpaceAround,
    SpaceEvenly,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum AlignItems {
    FlexStart,
    FlexEnd,
    Center,
    Baseline,
    Stretch,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum PositionType {
    Relative,
    Absolute,
    Fixed,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Size {
    pub width: Option<f32>,
    pub height: Option<f32>,
}

impl Default for UiLayoutComponent {
    fn default() -> Self {
        Self {
            bounds: ElementBounds { x: 0.0, y: 0.0, width: 0.0, height: 0.0 },
            padding: [0.0; 4],
            margin: [0.0; 4],
            layout_type: LayoutType::Flexbox,
            flex_direction: FlexDirection::Row,
            justify_content: JustifyContent::FlexStart,
            align_items: AlignItems::FlexStart,
            position_type: PositionType::Relative,
            size: Size { width: None, height: None },
            min_size: Size { width: None, height: None },
            max_size: Size { width: None, height: None },
        }
    }
}

impl Component for UiLayoutComponent {
    fn component_id() -> ComponentId {
        ComponentId(101)
    }
    
    fn component_name() -> &'static str {
        "UiLayoutComponent"
    }
    
    fn serialize(&self) -> Result<Vec<u8>, EcsError> {
        bincode::serialize(self)
            .map_err(|e| EcsError::SerializationFailed(e.to_string()))
    }
    
    fn deserialize(data: &[u8]) -> Result<Self, EcsError> {
        bincode::deserialize(data)
            .map_err(|e| EcsError::DeserializationFailed(e.to_string()))
    }
}

// Style component
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiStyleComponent {
    pub background_color: Option<Vector4<f32>>,
    pub text_color: Option<Vector4<f32>>,
    pub border_color: Option<Vector4<f32>>,
    pub border_width: f32,
    pub border_radius: f32,
    pub opacity: f32,
    pub font_size: f32,
    pub font_family: Option<String>,
    pub font_weight: FontWeight,
    pub text_align: TextAlign,
    pub visible: bool,
    pub z_index: i32,
    pub cursor: Option<String>,
    pub custom_styles: HashMap<String, String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum FontWeight {
    Light,
    Normal,
    Bold,
    ExtraBold,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum TextAlign {
    Left,
    Center,
    Right,
    Justify,
}

impl Default for UiStyleComponent {
    fn default() -> Self {
        Self {
            background_color: None,
            text_color: None,
            border_color: None,
            border_width: 0.0,
            border_radius: 0.0,
            opacity: 1.0,
            font_size: 14.0,
            font_family: None,
            font_weight: FontWeight::Normal,
            text_align: TextAlign::Left,
            visible: true,
            z_index: 0,
            cursor: None,
            custom_styles: HashMap::new(),
        }
    }
}

impl Component for UiStyleComponent {
    fn component_id() -> ComponentId {
        ComponentId(102)
    }
    
    fn component_name() -> &'static str {
        "UiStyleComponent"
    }
    
    fn serialize(&self) -> Result<Vec<u8>, EcsError> {
        bincode::serialize(self)
            .map_err(|e| EcsError::SerializationFailed(e.to_string()))
    }
    
    fn deserialize(data: &[u8]) -> Result<Self, EcsError> {
        bincode::deserialize(data)
            .map_err(|e| EcsError::DeserializationFailed(e.to_string()))
    }
}

// Input component
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiInputComponent {
    pub accepts_input: bool,
    pub tab_index: Option<i32>,
    pub on_click: Option<String>,
    pub on_hover: Option<String>,
    pub on_focus: Option<String>,
    pub on_blur: Option<String>,
    pub on_key_down: Option<String>,
    pub on_key_up: Option<String>,
}

impl Default for UiInputComponent {
    fn default() -> Self {
        Self {
            accepts_input: false,
            tab_index: None,
            on_click: None,
            on_hover: None,
            on_focus: None,
            on_blur: None,
            on_key_down: None,
            on_key_up: None,
        }
    }
}

impl Component for UiInputComponent {
    fn component_id() -> ComponentId {
        ComponentId(103)
    }
    
    fn component_name() -> &'static str {
        "UiInputComponent"
    }
    
    fn serialize(&self) -> Result<Vec<u8>, EcsError> {
        bincode::serialize(self)
            .map_err(|e| EcsError::SerializationFailed(e.to_string()))
    }
    
    fn deserialize(data: &[u8]) -> Result<Self, EcsError> {
        bincode::deserialize(data)
            .map_err(|e| EcsError::DeserializationFailed(e.to_string()))
    }
}

// Text component
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiTextComponent {
    pub text: String,
    pub selectable: bool,
    pub editable: bool,
    pub selection_start: Option<usize>,
    pub selection_end: Option<usize>,
    pub cursor_position: usize,
}

impl UiTextComponent {
    pub fn new(text: String) -> Self {
        Self {
            text,
            selectable: true,
            editable: false,
            selection_start: None,
            selection_end: None,
            cursor_position: 0,
        }
    }
}

impl Component for UiTextComponent {
    fn component_id() -> ComponentId {
        ComponentId(104)
    }
    
    fn component_name() -> &'static str {
        "UiTextComponent"
    }
    
    fn serialize(&self) -> Result<Vec<u8>, EcsError> {
        bincode::serialize(self)
            .map_err(|e| EcsError::SerializationFailed(e.to_string()))
    }
    
    fn deserialize(data: &[u8]) -> Result<Self, EcsError> {
        bincode::deserialize(data)
            .map_err(|e| EcsError::DeserializationFailed(e.to_string()))
    }
}