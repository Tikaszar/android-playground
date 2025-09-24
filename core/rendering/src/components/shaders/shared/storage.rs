//! Shader storage component - OPTIONAL (shaders feature)

#[cfg(feature = "shaders")]
use std::collections::HashMap;
#[cfg(feature = "shaders")]
use serde::{Serialize, Deserialize};
#[cfg(feature = "shaders")]
use playground_core_ecs::impl_component_data;
#[cfg(feature = "shaders")]
use crate::types::ResourceId;
#[cfg(feature = "shaders")]
use crate::resources::ShaderInfo;

/// Storage for compiled shaders
#[cfg(feature = "shaders")]
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ShaderStorage {
    pub shaders: HashMap<ResourceId, ShaderInfo>,
}

#[cfg(feature = "shaders")]
impl_component_data!(ShaderStorage);