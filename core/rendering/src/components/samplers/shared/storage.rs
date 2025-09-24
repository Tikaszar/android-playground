//! Sampler storage component - OPTIONAL (samplers feature)

#[cfg(feature = "samplers")]
use std::collections::HashMap;
#[cfg(feature = "samplers")]
use serde::{Serialize, Deserialize};
#[cfg(feature = "samplers")]
use playground_core_ecs::impl_component_data;
#[cfg(feature = "samplers")]
use crate::types::ResourceId;
#[cfg(feature = "samplers")]
use crate::resources::SamplerInfo;

/// Storage for texture samplers
#[cfg(feature = "samplers")]
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SamplerStorage {
    pub samplers: HashMap<ResourceId, SamplerInfo>,
}

#[cfg(feature = "samplers")]
impl_component_data!(SamplerStorage);