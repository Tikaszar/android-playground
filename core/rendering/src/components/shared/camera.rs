//! Camera component - defines a view into the scene

use serde::{Serialize, Deserialize};
use playground_core_ecs::impl_component_data;
use crate::types::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Camera {
    pub active: bool,
    pub render_order: Int,
    pub viewport: Option<Viewport>,
    pub clear_color: Option<ColorRGBA>,
    pub clear_depth: bool,
    pub clear_stencil: bool,
    pub render_layers: UInt,
}

impl Default for Camera {
    fn default() -> Self {
        Self {
            active: true,
            render_order: 0,
            viewport: None,
            clear_color: Some([0.0, 0.0, 0.0, 1.0]),
            clear_depth: true,
            clear_stencil: false,
            render_layers: 0xFFFFFFFF,
        }
    }
}

impl_component_data!(Camera);