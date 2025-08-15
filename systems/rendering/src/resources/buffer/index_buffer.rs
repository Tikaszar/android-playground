use crate::resources::buffer::BufferHandle;
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum IndexType {
    U16,
    U32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexBuffer {
    pub handle: BufferHandle,
    pub index_type: IndexType,
    pub index_count: u32,
}

impl IndexBuffer {
    pub fn new(handle: BufferHandle, index_type: IndexType, index_count: u32) -> Self {
        Self {
            handle,
            index_type,
            index_count,
        }
    }
}