use crate::resources::{ShaderHandle, VertexFormat, TextureFormat};
use crate::state::{BlendState, DepthStencilState, RasterizerState};
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PrimitiveTopology {
    Points,
    Lines,
    LineStrip,
    Triangles,
    TriangleStrip,
    TriangleFan,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PipelineDesc {
    pub vertex_shader: ShaderHandle,
    pub fragment_shader: ShaderHandle,
    pub vertex_format: VertexFormat,
    pub blend_state: BlendState,
    pub depth_stencil_state: DepthStencilState,
    pub rasterizer_state: RasterizerState,
    pub primitive_topology: PrimitiveTopology,
    pub render_target_formats: Vec<TextureFormat>,
}