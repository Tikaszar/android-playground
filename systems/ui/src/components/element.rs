use playground_core_ecs::{ComponentData, ComponentId, EcsError, EcsResult, EntityId};
use serde::{Serialize, Deserialize};
use async_trait::async_trait;
use bytes::Bytes;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiElementComponent {
    pub id: String,
    pub element_type: String,
    pub text_content: Option<String>,
    pub visible: bool,
    pub hovered: bool,
    pub focused: bool,
    pub disabled: bool,
    pub children: Vec<EntityId>,
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

#[async_trait]
impl ComponentData for UiElementComponent {
    fn component_id() -> ComponentId {
        "UiElementComponent".to_string()
    }
    
    fn component_name() -> &'static str {
        "UiElementComponent"
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