use playground_core_ecs::{ComponentData, ComponentId, EcsError, EcsResult};
use serde::{Serialize, Deserialize};
use nalgebra::Vector4;
use std::collections::HashMap;
use async_trait::async_trait;
use bytes::Bytes;

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

#[async_trait]
impl ComponentData for UiStyleComponent {
    fn component_id() -> ComponentId {
        std::any::TypeId::of::<Self>()
    }
    
    fn component_name() -> &'static str {
        "UiStyleComponent"
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