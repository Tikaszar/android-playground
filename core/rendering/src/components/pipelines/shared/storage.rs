//! Pipeline storage component - OPTIONAL (pipelines feature)

#[cfg(feature = "pipelines")]
use std::collections::HashMap;
#[cfg(feature = "pipelines")]
use serde::{Serialize, Deserialize};
#[cfg(feature = "pipelines")]
use playground_core_ecs::impl_component_data;
#[cfg(feature = "pipelines")]
use crate::types::ResourceId;
#[cfg(feature = "pipelines")]
use crate::resources::PipelineInfo;

/// Storage for render pipelines and current pipeline tracking
#[cfg(feature = "pipelines")]
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PipelineStorage {
    pub pipelines: HashMap<ResourceId, PipelineInfo>,
    pub current_pipeline: Option<ResourceId>,
}

#[cfg(feature = "pipelines")]
impl_component_data!(PipelineStorage);