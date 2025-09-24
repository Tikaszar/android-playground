//! Renderer backend state component - MANDATORY

use serde::{Serialize, Deserialize};
use playground_core_ecs::impl_component_data;

/// Tracks the active rendering backend and initialization state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RendererBackend {
    pub active_backend: String,
    pub is_initialized: bool,
}

impl Default for RendererBackend {
    fn default() -> Self {
        Self {
            active_backend: String::new(),
            is_initialized: false,
        }
    }
}

impl_component_data!(RendererBackend);