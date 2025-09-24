//! Material component

#[cfg(feature = "shaders")]
use std::collections::HashMap;
#[cfg(feature = "shaders")]
use serde::{Serialize, Deserialize};
#[cfg(feature = "shaders")]
use playground_core_ecs::{EntityRef, impl_component_data};
#[cfg(feature = "shaders")]
use crate::types::*;

#[cfg(feature = "shaders")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Material {
    pub shader: EntityRef,
    pub textures: HashMap<String, EntityRef>,
    pub uniforms: HashMap<String, UniformValue>,
    pub blend_mode: BlendMode,
    pub cull_mode: CullMode,
    pub depth_test: bool,
    pub depth_write: bool,
}

#[cfg(feature = "shaders")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum UniformValue {
    Float(Float),
    Float2(Vec2),
    Float3(Vec3),
    Float4(Vec4),
    Int(Int),
    Int2([Int; 2]),
    Int3([Int; 3]),
    Int4([Int; 4]),
    Mat2(Mat2),
    Mat3(Mat3),
    Mat4(Mat4),
}

#[cfg(feature = "shaders")]
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum BlendMode {
    None,
    Alpha,
    Additive,
    Multiply,
}

#[cfg(feature = "shaders")]
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum CullMode {
    None,
    Front,
    Back,
}

#[cfg(feature = "shaders")]
impl_component_data!(Material);