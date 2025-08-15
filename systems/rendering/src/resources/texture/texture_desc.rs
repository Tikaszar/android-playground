use crate::resources::texture::TextureFormat;
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TextureType {
    Texture2D,
    Texture3D,
    TextureCube,
    Texture2DArray,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TextureUsage {
    ShaderRead,
    RenderTarget,
    Storage,
    TransferSrc,
    TransferDst,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextureDesc {
    pub texture_type: TextureType,
    pub format: TextureFormat,
    pub width: u32,
    pub height: u32,
    pub depth: u32,
    pub mip_levels: u32,
    pub array_layers: u32,
    pub usage: Vec<TextureUsage>,
    pub sample_count: u32,
}

impl Default for TextureDesc {
    fn default() -> Self {
        Self {
            texture_type: TextureType::Texture2D,
            format: TextureFormat::RGBA8,
            width: 1,
            height: 1,
            depth: 1,
            mip_levels: 1,
            array_layers: 1,
            usage: vec![TextureUsage::ShaderRead],
            sample_count: 1,
        }
    }
}