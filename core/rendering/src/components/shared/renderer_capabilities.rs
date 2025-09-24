//! Renderer capabilities component - MANDATORY

use serde::{Serialize, Deserialize};
use playground_core_ecs::impl_component_data;
use crate::types::RendererCapabilities;

/// Wraps the renderer capabilities in a component
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RendererCapabilitiesComponent(pub RendererCapabilities);

impl Default for RendererCapabilitiesComponent {
    fn default() -> Self {
        Self(RendererCapabilities::default())
    }
}

impl_component_data!(RendererCapabilitiesComponent);