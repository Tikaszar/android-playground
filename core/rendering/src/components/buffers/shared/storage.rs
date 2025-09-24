//! Buffer storage component - OPTIONAL (buffers feature)

#[cfg(feature = "buffers")]
use std::collections::HashMap;
#[cfg(feature = "buffers")]
use serde::{Serialize, Deserialize};
#[cfg(feature = "buffers")]
use playground_core_ecs::impl_component_data;
#[cfg(feature = "buffers")]
use crate::types::ResourceId;
#[cfg(feature = "buffers")]
use crate::resources::BufferInfo;

/// Storage for GPU buffers (vertex, index, etc.)
#[cfg(feature = "buffers")]
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct BufferStorage {
    pub buffers: HashMap<ResourceId, BufferInfo>,
}

#[cfg(feature = "buffers")]
impl_component_data!(BufferStorage);