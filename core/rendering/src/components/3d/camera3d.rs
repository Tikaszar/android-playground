//! 3D Camera projection component

use serde::{Serialize, Deserialize};
use playground_core_ecs::impl_component_data;
use crate::types::*;

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Camera3D {
    pub projection: ProjectionType,
    pub fov: Float,
    pub ortho_size: Float,
    pub aspect: Float,
    pub near: Float,
    pub far: Float,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum ProjectionType {
    Perspective,
    Orthographic,
}

impl Default for Camera3D {
    fn default() -> Self {
        Self {
            projection: ProjectionType::Perspective,
            fov: 60.0_f32.to_radians(),
            ortho_size: 10.0,
            aspect: 16.0 / 9.0,
            near: 0.1,
            far: 1000.0,
        }
    }
}

impl_component_data!(Camera3D);