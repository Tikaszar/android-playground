//! Texture resource component

#[cfg(feature = "textures")]
use serde::{Serialize, Deserialize};
#[cfg(feature = "textures")]
use playground_core_ecs::impl_component_data;
#[cfg(feature = "textures")]
use crate::types::*;

#[cfg(feature = "textures")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Texture {
    pub gpu_resource_id: ResourceId,
    pub width: UInt,
    pub height: UInt,
    pub format: TextureFormat,
    pub mip_levels: UInt,
}

#[cfg(feature = "textures")]
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum TextureFormat {
    R8,
    RG8,
    RGB8,
    RGBA8,
    R16F,
    RG16F,
    RGB16F,
    RGBA16F,
    R32F,
    RG32F,
    RGB32F,
    RGBA32F,
    Depth24,
    Depth32F,
    Depth24Stencil8,
    SRGBA8,
}

#[cfg(feature = "textures")]
impl_component_data!(Texture);