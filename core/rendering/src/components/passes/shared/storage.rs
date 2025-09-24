//! Render pass storage component - OPTIONAL (passes feature)

#[cfg(feature = "passes")]
use std::collections::HashMap;
#[cfg(feature = "passes")]
use serde::{Serialize, Deserialize};
#[cfg(feature = "passes")]
use playground_core_ecs::impl_component_data;
#[cfg(feature = "passes")]
use crate::types::ResourceId;
#[cfg(feature = "passes")]
use crate::resources::RenderPassInfo;

/// Storage for render passes and active pass tracking
#[cfg(feature = "passes")]
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RenderPassStorage {
    pub render_passes: HashMap<ResourceId, RenderPassInfo>,
    pub active_pass: Option<ResourceId>,
}

#[cfg(feature = "passes")]
impl_component_data!(RenderPassStorage);