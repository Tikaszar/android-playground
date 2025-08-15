use crate::resources::buffer::BufferHandle;
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UniformBuffer {
    pub handle: BufferHandle,
    pub size: usize,
}

impl UniformBuffer {
    pub fn new(handle: BufferHandle, size: usize) -> Self {
        Self { handle, size }
    }
}