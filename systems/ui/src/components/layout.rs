use playground_core_ecs::{Component, ComponentId, EcsError, EcsResult};
use serde::{Serialize, Deserialize};
use async_trait::async_trait;
use bytes::Bytes;

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

#[async_trait]
impl Component for UiLayoutComponent {
    fn component_id() -> ComponentId {
        std::any::TypeId::of::<Self>()
    }
    
    fn component_name() -> &'static str {
        "UiLayoutComponent"
    }
    
    async fn serialize(&self) -> EcsResult<Bytes> {
        let data = bincode::serialize(self)
            .map_err(|e| EcsError::SerializationFailed(e.to_string()))?;
        Ok(Bytes::from(data))
    }
    
    async fn deserialize(bytes: &Bytes) -> EcsResult<Self> {
        bincode::deserialize(bytes)
            .map_err(|e| EcsError::DeserializationFailed(e.to_string()))
    }
}