//! Shader resource component

#[cfg(feature = "shaders")]
use serde::{Serialize, Deserialize};
#[cfg(feature = "shaders")]
use playground_core_ecs::impl_component_data;
#[cfg(feature = "shaders")]
use crate::types::*;

#[cfg(feature = "shaders")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Shader {
    pub gpu_resource_id: ResourceId,
    pub shader_type: ShaderType,
    pub entry_point: String,
}

#[cfg(feature = "shaders")]
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum ShaderType {
    Vertex,
    Fragment,
    Compute,
    Geometry,
    TessellationControl,
    TessellationEvaluation,
}

#[cfg(feature = "shaders")]
impl_component_data!(Shader);