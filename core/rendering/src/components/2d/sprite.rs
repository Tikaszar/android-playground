//! Sprite component for 2D rendering

use serde::{Serialize, Deserialize};
use playground_core_ecs::{EntityId, impl_component_data};
use crate::types::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Sprite {
    pub texture: Option<EntityId>,
    pub color: ColorRGBA,
    pub flip_x: bool,
    pub flip_y: bool,
    pub source_rect: Option<Rect>,
    pub sort_order: Int,
}

impl Default for Sprite {
    fn default() -> Self {
        Self {
            texture: None,
            color: [1.0, 1.0, 1.0, 1.0],
            flip_x: false,
            flip_y: false,
            source_rect: None,
            sort_order: 0,
        }
    }
}

impl_component_data!(Sprite);