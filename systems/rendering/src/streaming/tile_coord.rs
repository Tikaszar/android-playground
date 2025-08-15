use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TileCoord {
    pub x: u32,
    pub y: u32,
    pub z: u32,
    pub lod: u32,
}

impl TileCoord {
    pub fn new(x: u32, y: u32, z: u32, lod: u32) -> Self {
        Self { x, y, z, lod }
    }
}