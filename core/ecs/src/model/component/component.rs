//! Component wrapper type

use bytes::Bytes;
use crate::model::component::ComponentId;
use crate::EcsResult;

/// Concrete Component struct - holds serialized component data
///
/// Users manually serialize their data and create Components.
/// Helper functions are provided for common serialization patterns.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
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
    /// Create a new component from raw serialized data
    pub fn new(data: Bytes, component_id: ComponentId, component_name: String) -> Self {
        let size_hint = data.len();
        Self {
            data,
            component_id,
            component_name,
            size_hint,
        }
    }

    /// Helper: Create a component from any type that implements Serialize
    ///
    /// This is a convenience function for the common pattern of using bincode serialization.
    pub fn from_serializable<T: serde::Serialize + 'static>(value: &T) -> EcsResult<Self> {
        let data = bincode::serialize(value)
            .map_err(|e| crate::EcsError::SerializationError(e.to_string()))?;

        let size_hint = data.len();

        Ok(Self {
            data: Bytes::from(data),
            component_id: ComponentId::from_type_name::<T>(),
            component_name: std::any::type_name::<T>().to_string(),
            size_hint,
        })
    }

    /// Helper: Deserialize this component to a specific type
    ///
    /// This is a convenience function for the common pattern of using bincode deserialization.
    pub fn to_deserializable<T: serde::de::DeserializeOwned>(&self) -> EcsResult<T> {
        bincode::deserialize(&self.data)
            .map_err(|e| crate::EcsError::DeserializationError(e.to_string()))
    }

    /// Check if this component is of a specific type
    pub fn is_type<T: 'static>(&self) -> bool {
        self.component_id == ComponentId::from_type_name::<T>()
    }

    /// Get the component type name
    pub fn type_name(&self) -> &str {
        &self.component_name
    }

    /// Get the size of the serialized data
    pub fn size(&self) -> usize {
        self.size_hint
    }
}