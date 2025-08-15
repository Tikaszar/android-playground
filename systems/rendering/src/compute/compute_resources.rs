use crate::resources::{StorageBuffer, TextureHandle};
use std::collections::HashMap;

pub struct ComputeResources {
    pub storage_buffers: HashMap<u32, StorageBuffer>,
    pub textures: HashMap<u32, TextureHandle>,
}

impl ComputeResources {
    pub fn new() -> Self {
        Self {
            storage_buffers: HashMap::new(),
            textures: HashMap::new(),
        }
    }
    
    pub fn bind_storage_buffer(&mut self, slot: u32, buffer: StorageBuffer) {
        self.storage_buffers.insert(slot, buffer);
    }
    
    pub fn bind_texture(&mut self, slot: u32, texture: TextureHandle) {
        self.textures.insert(slot, texture);
    }
}