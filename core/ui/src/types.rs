use serde::{Serialize, Deserialize};
use uuid::Uuid;

/// Unique identifier for UI elements
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ElementId(pub Uuid);

impl ElementId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

impl Default for ElementId {
    fn default() -> Self {
        Self::new()
    }
}

/// Type of UI element
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ElementType {
    Panel,
    Text,
    Button,
    Input,
    Image,
    ScrollView,
    List,
    Grid,
    Canvas,
    Custom,
}

/// Style properties for UI elements
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Style {
    pub background_color: Option<[f32; 4]>,
    pub text_color: Option<[f32; 4]>,
    pub border_color: Option<[f32; 4]>,
    pub border_width: f32,
    pub border_radius: f32,
    pub padding: [f32; 4], // top, right, bottom, left
    pub margin: [f32; 4],   // top, right, bottom, left
    pub opacity: f32,
    pub visible: bool,
    pub z_index: i32,
    pub font_size: Option<f32>,
    pub font_family: Option<String>,
    pub font_weight: Option<FontWeight>,
    pub text_align: Option<TextAlign>,
    pub overflow: Overflow,
}

impl Default for Style {
    fn default() -> Self {
        Self {
            background_color: None,
            text_color: Some([0.0, 0.0, 0.0, 1.0]),
            border_color: None,
            border_width: 0.0,
            border_radius: 0.0,
            padding: [0.0; 4],
            margin: [0.0; 4],
            opacity: 1.0,
            visible: true,
            z_index: 0,
            font_size: Some(14.0),
            font_family: Some("sans-serif".to_string()),
            font_weight: Some(FontWeight::Normal),
            text_align: Some(TextAlign::Left),
            overflow: Overflow::Visible,
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

/// Text alignment
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum TextAlign {
    Left,
    Center,
    Right,
    Justify,
}

/// Overflow behavior
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum Overflow {
    Visible,
    Hidden,
    Scroll,
    Auto,
}

/// Element bounds in 2D space
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Bounds {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

impl Bounds {
    pub fn new(x: f32, y: f32, width: f32, height: f32) -> Self {
        Self { x, y, width, height }
    }
    
    pub fn contains(&self, x: f32, y: f32) -> bool {
        x >= self.x && x <= self.x + self.width &&
        y >= self.y && y <= self.y + self.height
    }
    
    pub fn intersects(&self, other: &Bounds) -> bool {
        !(self.x + self.width < other.x ||
          other.x + other.width < self.x ||
          self.y + self.height < other.y ||
          other.y + other.height < self.y)
    }
}

/// Layout type for container elements
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum LayoutType {
    None,
    Absolute,
    Flexbox,
    Grid,
    Stack,
}

/// Flexbox layout properties
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FlexLayout {
    pub direction: FlexDirection,
    pub justify_content: JustifyContent,
    pub align_items: AlignItems,
    pub align_content: AlignContent,
    pub flex_wrap: FlexWrap,
    pub gap: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum FlexDirection {
    Row,
    RowReverse,
    Column,
    ColumnReverse,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum JustifyContent {
    Start,
    End,
    Center,
    SpaceBetween,
    SpaceAround,
    SpaceEvenly,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum AlignItems {
    Start,
    End,
    Center,
    Stretch,
    Baseline,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum AlignContent {
    Start,
    End,
    Center,
    SpaceBetween,
    SpaceAround,
    Stretch,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum FlexWrap {
    NoWrap,
    Wrap,
    WrapReverse,
}

/// Grid layout properties
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GridLayout {
    pub columns: Vec<GridTrack>,
    pub rows: Vec<GridTrack>,
    pub gap: f32,
    pub justify_items: JustifyContent,
    pub align_items: AlignItems,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GridTrack {
    Fixed(f32),
    Fraction(f32),
    Auto,
    MinMax(f32, f32),
}