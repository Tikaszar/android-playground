use playground_core_ecs::{ComponentData, ComponentId, EcsError, EcsResult};
use serde::{Serialize, Deserialize};
use async_trait::async_trait;
use bytes::Bytes;

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

#[async_trait]
#[async_trait]
impl ComponentData for UiTextComponent {
    fn component_id() -> ComponentId {
        std::any::TypeId::of::<Self>()
    }
    
    fn component_name() -> &'static str {
        "UiTextComponent"
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