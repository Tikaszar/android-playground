//! Uniform buffer storage component - OPTIONAL (uniforms feature)

#[cfg(feature = "uniforms")]
use std::collections::HashMap;
#[cfg(feature = "uniforms")]
use serde::{Serialize, Deserialize};
#[cfg(feature = "uniforms")]
use playground_core_ecs::impl_component_data;
#[cfg(feature = "uniforms")]
use crate::types::ResourceId;
#[cfg(feature = "uniforms")]
use crate::resources::UniformBufferInfo;

/// Storage for uniform buffers
#[cfg(feature = "uniforms")]
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct UniformBufferStorage {
    pub uniform_buffers: HashMap<ResourceId, UniformBufferInfo>,
}

#[cfg(feature = "uniforms")]
impl_component_data!(UniformBufferStorage);