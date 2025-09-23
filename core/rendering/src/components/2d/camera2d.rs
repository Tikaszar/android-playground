//! 2D Camera projection component

use serde::{Serialize, Deserialize};
use playground_core_ecs::impl_component_data;
use crate::types::*;

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Camera2D {
    pub size: Vec2,
    pub near: Float,
    pub far: Float,
}

impl Default for Camera2D {
    fn default() -> Self {
        Self {
            size: [10.0, 10.0],
            near: -100.0,
            far: 100.0,
        }
    }
}

impl_component_data!(Camera2D);