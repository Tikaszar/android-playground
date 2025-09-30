//! Component system for the ECS
//! 
//! Components are data attached to entities. We use a concrete Component struct
//! to avoid dyn trait objects, with serialization for type erasure.

use bytes::Bytes;
use serde::{Serialize, Deserialize};
use crate::CoreResult;

/// Component ID type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ComponentId(pub u32);

impl ComponentId {
    /// Create a component ID from a type name hash
    pub fn from_type_name<T: 'static>() -> Self {
        let type_name = std::any::type_name::<T>();
        let hash = Self::hash_type_name(type_name);
        ComponentId(hash)
    }
    
    /// Hash a type name to create a component ID
    fn hash_type_name(name: &str) -> u32 {
        // Simple FNV-1a hash
        let mut hash = 2166136261u32;
        for byte in name.bytes() {
            hash ^= byte as u32;
            hash = hash.wrapping_mul(16777619);
        }
        hash
    }
}

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
    pub async fn new<T: ComponentData>(data: T) -> CoreResult<Self> {
        let serialized = data.serialize().await?;
        Ok(Component {
            data: serialized.clone(),
            component_id: T::component_id(),
            component_name: T::component_name(),
            size_hint: serialized.len(),
        })
    }
    
    /// Deserialize the component back to its original type
    pub async fn deserialize<T: ComponentData>(&self) -> CoreResult<T> {
        T::deserialize(&self.data).await
    }
    
    /// Check if this component is of a specific type
    pub fn is_type<T: ComponentData>(&self) -> bool {
        self.component_id == T::component_id()
    }
}

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
    async fn serialize(&self) -> CoreResult<Bytes>;
    
    /// Deserialize the component from bytes
    async fn deserialize(bytes: &Bytes) -> CoreResult<Self> where Self: Sized;
}

/// Macro to implement ComponentData for types that implement Serialize + Deserialize
#[macro_export]
macro_rules! impl_component_data {
    ($type:ty) => {
        impl $crate::ComponentData for $type {
            fn component_id() -> $crate::ComponentId {
                $crate::ComponentId::from_type_name::<Self>()
            }
            
            async fn serialize(&self) -> $crate::CoreResult<bytes::Bytes> {
                let bytes = bincode::serialize(self)
                    .map_err(|e| $crate::CoreError::SerializationError(e.to_string()))?;
                Ok(bytes::Bytes::from(bytes))
            }
            
            async fn deserialize(bytes: &bytes::Bytes) -> $crate::CoreResult<Self> {
                bincode::deserialize(bytes)
                    .map_err(|e| $crate::CoreError::DeserializationError(e.to_string()))
            }
        }
    };
}