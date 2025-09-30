//! Component wrapper type

use bytes::Bytes;
use crate::model::component::ComponentId;
use crate::EcsResult;

/// Concrete Component struct - this is what we store, NOT dyn ComponentData!
#[derive(Debug, Clone)]
pub struct Component {
    /// The serialized component data
    pub data: Bytes,

    /// The component type ID
    pub component_id: ComponentId,

    /// The component type name (for debugging)
    pub component_name: String,

    /// Size hint for allocation
    pub size_hint: usize,
}

impl Component {
    /// Create a new component from any ComponentData type
    pub async fn new<T: super::ComponentData>(data: T) -> EcsResult<Self> {
        let serialized = data.serialize().await?;
        Ok(Component {
            data: serialized.clone(),
            component_id: T::component_id(),
            component_name: T::component_name(),
            size_hint: serialized.len(),
        })
    }

    /// Deserialize the component back to its original type
    pub async fn deserialize<T: super::ComponentData>(&self) -> EcsResult<T> {
        T::deserialize(&self.data).await
    }

    /// Check if this component is of a specific type
    pub fn is_type<T: super::ComponentData>(&self) -> bool {
        self.component_id == T::component_id()
    }
}