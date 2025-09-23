//! Render layer component - which layer this entity renders on

use serde::{Serialize, Deserialize};
use playground_core_ecs::impl_component_data;
use crate::types::*;

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct RenderLayer {
    pub layer: UInt,
}

impl Default for RenderLayer {
    fn default() -> Self {
        Self {
            layer: 1,
        }
    }
}

impl_component_data!(RenderLayer);