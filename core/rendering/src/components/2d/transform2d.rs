//! 2D Transform component

use serde::{Serialize, Deserialize};
use playground_core_ecs::impl_component_data;
use crate::types::*;

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Transform2D {
    pub position: Vec2,
    pub rotation: Float,
    pub scale: Vec2,
}

impl Default for Transform2D {
    fn default() -> Self {
        Self {
            position: [0.0, 0.0],
            rotation: 0.0,
            scale: [1.0, 1.0],
        }
    }
}

impl_component_data!(Transform2D);