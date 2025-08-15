use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TextureFormat {
    // Color formats
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
    
    // Depth/Stencil formats
    Depth16,
    Depth24,
    Depth32F,
    Depth24Stencil8,
    Depth32FStencil8,
    
    // Compressed formats
    DXT1,
    DXT3,
    DXT5,
    Etc2Rgb,
    Etc2Rgba,
    Astc4x4,
    Astc8x8,
}

impl TextureFormat {
    pub fn bytes_per_pixel(&self) -> usize {
        match self {
            Self::R8 => 1,
            Self::RG8 => 2,
            Self::RGB8 => 3,
            Self::RGBA8 => 4,
            Self::R16F => 2,
            Self::RG16F => 4,
            Self::RGB16F => 6,
            Self::RGBA16F => 8,
            Self::R32F => 4,
            Self::RG32F => 8,
            Self::RGB32F => 12,
            Self::RGBA32F => 16,
            Self::Depth16 => 2,
            Self::Depth24 => 3,
            Self::Depth32F => 4,
            Self::Depth24Stencil8 => 4,
            Self::Depth32FStencil8 => 5,
            _ => 0, // Compressed formats
        }
    }
    
    pub fn is_compressed(&self) -> bool {
        matches!(self,
            Self::DXT1 | Self::DXT3 | Self::DXT5 |
            Self::Etc2Rgb | Self::Etc2Rgba |
            Self::Astc4x4 | Self::Astc8x8
        )
    }
    
    pub fn is_depth(&self) -> bool {
        matches!(self,
            Self::Depth16 | Self::Depth24 | Self::Depth32F |
            Self::Depth24Stencil8 | Self::Depth32FStencil8
        )
    }
}