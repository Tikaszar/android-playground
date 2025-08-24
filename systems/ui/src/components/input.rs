use playground_core_ecs::{ComponentData, ComponentId, EcsError, EcsResult};
use serde::{Serialize, Deserialize};
use async_trait::async_trait;
use bytes::Bytes;

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

#[async_trait]
impl ComponentData for UiInputComponent {
    fn component_id() -> ComponentId {
        std::any::TypeId::of::<Self>()
    }
    
    fn component_name() -> &'static str {
        "UiInputComponent"
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