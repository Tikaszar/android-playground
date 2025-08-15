use crate::resources::buffer::BufferHandle;
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageBuffer {
    pub handle: BufferHandle,
    pub size: usize,
    pub read_only: bool,
}

impl StorageBuffer {
    pub fn new(handle: BufferHandle, size: usize, read_only: bool) -> Self {
        Self {
            handle,
            size,
            read_only,
        }
    }
}