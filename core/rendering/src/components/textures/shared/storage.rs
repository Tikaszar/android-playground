//! Texture storage component - OPTIONAL (textures feature)

#[cfg(feature = "textures")]
use std::collections::HashMap;
#[cfg(feature = "textures")]
use serde::{Serialize, Deserialize};
#[cfg(feature = "textures")]
use playground_core_ecs::impl_component_data;
#[cfg(feature = "textures")]
use crate::types::ResourceId;
#[cfg(feature = "textures")]
use crate::resources::TextureInfo;

/// Storage for loaded textures
#[cfg(feature = "textures")]
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TextureStorage {
    pub textures: HashMap<ResourceId, TextureInfo>,
}

#[cfg(feature = "textures")]
impl_component_data!(TextureStorage);