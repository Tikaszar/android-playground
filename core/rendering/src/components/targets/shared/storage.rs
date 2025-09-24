//! Render target storage component - OPTIONAL (targets feature)

#[cfg(feature = "targets")]
use std::collections::HashMap;
#[cfg(feature = "targets")]
use serde::{Serialize, Deserialize};
#[cfg(feature = "targets")]
use playground_core_ecs::impl_component_data;
#[cfg(feature = "targets")]
use crate::types::ResourceId;
#[cfg(feature = "targets")]
use crate::resources::RenderTargetInfo;

/// Storage for render targets and current target tracking
#[cfg(feature = "targets")]
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RenderTargetStorage {
    pub targets: HashMap<ResourceId, RenderTargetInfo>,
    pub current_target: Option<ResourceId>,
}

#[cfg(feature = "targets")]
impl_component_data!(RenderTargetStorage);