//! 3D Transform component

use serde::{Serialize, Deserialize};
use playground_core_ecs::impl_component_data;
use crate::types::*;

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Transform3D {
    pub position: Vec3,
    pub rotation: Quat,
    pub scale: Vec3,
}

impl Default for Transform3D {
    fn default() -> Self {
        Self {
            position: [0.0, 0.0, 0.0],
            rotation: [0.0, 0.0, 0.0, 1.0],
            scale: [1.0, 1.0, 1.0],
        }
    }
}

impl_component_data!(Transform3D);