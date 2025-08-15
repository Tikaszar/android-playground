use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum VertexAttributeType {
    Float,
    Float2,
    Float3,
    Float4,
    Int,
    Int2,
    Int3,
    Int4,
    UInt,
    UInt2,
    UInt3,
    UInt4,
    Mat3,
    Mat4,
}

impl VertexAttributeType {
    pub fn size(&self) -> usize {
        match self {
            Self::Float | Self::Int | Self::UInt => 4,
            Self::Float2 | Self::Int2 | Self::UInt2 => 8,
            Self::Float3 | Self::Int3 | Self::UInt3 => 12,
            Self::Float4 | Self::Int4 | Self::UInt4 => 16,
            Self::Mat3 => 36,
            Self::Mat4 => 64,
        }
    }
    
    pub fn component_count(&self) -> u32 {
        match self {
            Self::Float | Self::Int | Self::UInt => 1,
            Self::Float2 | Self::Int2 | Self::UInt2 => 2,
            Self::Float3 | Self::Int3 | Self::UInt3 => 3,
            Self::Float4 | Self::Int4 | Self::UInt4 => 4,
            Self::Mat3 => 9,
            Self::Mat4 => 16,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VertexAttribute {
    pub name: String,
    pub location: u32,
    pub attribute_type: VertexAttributeType,
    pub offset: usize,
    pub normalized: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VertexFormat {
    pub attributes: Vec<VertexAttribute>,
    pub stride: usize,
}

impl VertexFormat {
    pub fn new(attributes: Vec<VertexAttribute>) -> Self {
        let stride = attributes.iter()
            .map(|a| a.attribute_type.size())
            .sum();
        
        Self {
            attributes,
            stride,
        }
    }
}