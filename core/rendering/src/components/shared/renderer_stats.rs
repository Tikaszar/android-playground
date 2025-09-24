//! Renderer statistics component - MANDATORY

use serde::{Serialize, Deserialize};
use playground_core_ecs::impl_component_data;
use crate::types::RendererStats;

/// Wraps the renderer statistics in a component
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RendererStatsComponent(pub RendererStats);

impl Default for RendererStatsComponent {
    fn default() -> Self {
        Self(RendererStats::default())
    }
}

impl_component_data!(RendererStatsComponent);