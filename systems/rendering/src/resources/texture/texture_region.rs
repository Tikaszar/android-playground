use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct TextureRegion {
    pub x: u32,
    pub y: u32,
    pub z: u32,
    pub width: u32,
    pub height: u32,
    pub depth: u32,
    pub mip_level: u32,
    pub array_layer: u32,
}

impl TextureRegion {
    pub fn new_2d(x: u32, y: u32, width: u32, height: u32) -> Self {
        Self {
            x,
            y,
            z: 0,
            width,
            height,
            depth: 1,
            mip_level: 0,
            array_layer: 0,
        }
    }
}