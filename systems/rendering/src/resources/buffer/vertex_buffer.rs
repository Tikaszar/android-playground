use crate::resources::buffer::{BufferHandle, VertexFormat};
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VertexBuffer {
    pub handle: BufferHandle,
    pub format: VertexFormat,
    pub vertex_count: u32,
}

impl VertexBuffer {
    pub fn new(handle: BufferHandle, format: VertexFormat, vertex_count: u32) -> Self {
        Self {
            handle,
            format,
            vertex_count,
        }
    }
}