//! Component data trait

use bytes::Bytes;
use crate::EcsResult;
use crate::model::component::ComponentId;

/// Trait for component data types
///
/// This trait is implemented by actual component types, but we never store
/// Box<dyn ComponentData>. We always store the concrete Component struct.
pub trait ComponentData: Send + Sync + 'static {
    /// Get the component ID for this type
    fn component_id() -> ComponentId where Self: Sized;

    /// Get the component name for debugging
    fn component_name() -> String where Self: Sized {
        std::any::type_name::<Self>().to_string()
    }

    /// Serialize the component to bytes
    async fn serialize(&self) -> EcsResult<Bytes>;

    /// Deserialize the component from bytes
    async fn deserialize(bytes: &Bytes) -> EcsResult<Self> where Self: Sized;
}

/// Macro to implement ComponentData for types that implement Serialize + Deserialize
#[macro_export]
macro_rules! impl_component_data {
    ($type:ty) => {
        impl $crate::model::component::ComponentData for $type {
            fn component_id() -> $crate::model::component::ComponentId {
                $crate::model::component::ComponentId::from_type_name::<Self>()
            }

            async fn serialize(&self) -> $crate::EcsResult<bytes::Bytes> {
                let bytes = bincode::serialize(self)
                    .map_err(|e| $crate::EcsError::SerializationError(e.to_string()))?;
                Ok(bytes::Bytes::from(bytes))
            }

            async fn deserialize(bytes: &bytes::Bytes) -> $crate::EcsResult<Self> {
                bincode::deserialize(bytes)
                    .map_err(|e| $crate::EcsError::DeserializationError(e.to_string()))
            }
        }
    };
}