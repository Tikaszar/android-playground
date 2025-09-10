//! Component contracts for the ECS

use bytes::Bytes;
use async_trait::async_trait;
use crate::error::EcsResult;

/// Component ID type - string-based to avoid TypeId
pub type ComponentId = String;

/// Trait that all component data types must implement
/// This is the contract for components in the ECS
#[async_trait]
pub trait ComponentData: Send + Sync + 'static {
    /// Get the unique component ID
    fn component_id() -> ComponentId where Self: Sized {
        Self::component_name().to_string()
    }
    
    /// Get the component name
    fn component_name() -> &'static str where Self: Sized {
        "UnknownComponent"
    }
    
    /// Serialize the component to bytes
    async fn serialize(&self) -> EcsResult<Bytes>;
    
    /// Deserialize the component from bytes
    async fn deserialize(bytes: &Bytes) -> EcsResult<Self> where Self: Sized;
}