//! Render target component

use crate::types::*;
use playground_core_ecs::impl_component_data;
use serde::{Deserialize, Serialize};

/// Render target as an ECS component
#[cfg(feature = "rendering")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RenderTargetComponent {
    pub target: RenderTarget,
}

#[cfg(feature = "rendering")]
impl_component_data!(RenderTargetComponent);

#[cfg(feature = "rendering")]
impl RenderTargetComponent {
    pub fn new(id: UInt, width: UInt, height: UInt) -> Self {
        Self {
            target: RenderTarget {
                id,
                width,
                height,
                scale_factor: 1.0,
                is_primary: false,
                properties: std::collections::HashMap::new(),
            },
        }
    }

    pub fn resize(&mut self, width: UInt, height: UInt) {
        self.target.width = width;
        self.target.height = height;
    }

    pub fn set_scale_factor(&mut self, scale: Float) {
        self.target.scale_factor = scale;
    }
}