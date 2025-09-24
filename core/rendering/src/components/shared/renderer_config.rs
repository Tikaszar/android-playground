//! Renderer configuration component - MANDATORY

use serde::{Serialize, Deserialize};
use playground_core_ecs::impl_component_data;
use crate::types::RendererConfig;

/// Wraps the renderer configuration in a component
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RendererConfigComponent(pub RendererConfig);

impl_component_data!(RendererConfigComponent);