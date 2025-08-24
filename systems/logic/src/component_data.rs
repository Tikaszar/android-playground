use bytes::Bytes;
use serde::{Serialize, Deserialize};
use std::any::TypeId;
use crate::error::{LogicResult, LogicError};
use crate::component::Component;

/// Concrete wrapper for component data that avoids Box<dyn Any>
#[derive(Clone)]
pub struct ComponentData {
    data: Bytes,
    type_id: TypeId,
    type_name: String,
}

impl ComponentData {
    /// Create new ComponentData from a component
    pub fn new<T: Component + Serialize>(component: T) -> LogicResult<Self> {
        let data = bincode::serialize(&component)
            .map_err(|e| LogicError::SerializationError(e.to_string()))?
            .into();
        
        Ok(Self {
            data,
            type_id: TypeId::of::<T>(),
            type_name: T::type_name().to_string(),
        })
    }
    
    /// Create from raw bytes (used internally)
    pub fn from_bytes(data: Bytes, type_id: TypeId, type_name: String) -> Self {
        Self {
            data,
            type_id,
            type_name,
        }
    }
    
    /// Get the type ID of this component
    pub fn type_id(&self) -> TypeId {
        self.type_id
    }
    
    /// Get the type name of this component
    pub fn type_name(&self) -> &str {
        &self.type_name
    }
    
    /// Get the raw bytes
    pub fn as_bytes(&self) -> &Bytes {
        &self.data
    }
    
    /// Deserialize back to the original component type
    pub fn deserialize<T: for<'de> Deserialize<'de>>(&self) -> LogicResult<T> {
        bincode::deserialize(&self.data)
            .map_err(|e| LogicError::DeserializationError(e.to_string()))
    }
    
    /// Check if this is a specific component type
    pub fn is<T: Component>(&self) -> bool {
        self.type_id == TypeId::of::<T>()
    }
    
    /// Get size hint for memory tracking
    pub fn size_hint(&self) -> usize {
        self.data.len()
    }
}

/// Helper to convert a tuple for spawn_with pattern
impl From<(TypeId, ComponentData)> for ComponentData {
    fn from((_, data): (TypeId, ComponentData)) -> Self {
        data
    }
}