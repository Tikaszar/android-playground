
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TextureFormat {
    RGBA8,
    RGB8,
    RG8,
    R8,
    RGBA16F,
    RGB16F,
    RG16F,
    R16F,
    RGBA32F,
    RGB32F,
    RG32F,
    R32F,
    Depth24Stencil8,
    Depth32F,
}

impl TextureFormat {
    pub fn bytes_per_pixel(&self) -> usize {
        match self {
            Self::RGBA8 => 4,
            Self::RGB8 => 3,
            Self::RG8 => 2,
            Self::R8 => 1,
            Self::RGBA16F => 8,
            Self::RGB16F => 6,
            Self::RG16F => 4,
            Self::R16F => 2,
            Self::RGBA32F => 16,
            Self::RGB32F => 12,
            Self::RG32F => 8,
            Self::R32F => 4,
            Self::Depth24Stencil8 => 4,
            Self::Depth32F => 4,
        }
    }
}

pub struct Texture2D {
    texture_id: u32,
    width: u32,
    height: u32,
    format: TextureFormat,
    mipmaps: bool,
}

impl Texture2D {
    pub fn new(width: u32, height: u32, format: TextureFormat) -> Self {
        Self {
            texture_id: 0,
            width,
            height,
            format,
            mipmaps: false,
        }
    }

    pub fn with_mipmaps(mut self) -> Self {
        self.mipmaps = true;
        self
    }

    pub fn upload(&mut self, data: &[u8]) -> Result<(), String> {
        let expected_size = (self.width * self.height) as usize * self.format.bytes_per_pixel();
        if data.len() != expected_size {
            return Err(format!(
                "Texture data size mismatch: expected {}, got {}",
                expected_size,
                data.len()
            ));
        }
        
        Ok(())
    }

    pub fn bind(&self, texture_unit: u32) {
    }

    pub fn unbind(&self) {
    }

    pub fn generate_mipmaps(&mut self) {
        self.mipmaps = true;
    }

    pub fn width(&self) -> u32 {
        self.width
    }

    pub fn height(&self) -> u32 {
        self.height
    }

    pub fn format(&self) -> TextureFormat {
        self.format
    }

    pub fn id(&self) -> u32 {
        self.texture_id
    }
}