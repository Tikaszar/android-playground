//! Command buffer storage component - OPTIONAL (commands feature)

#[cfg(feature = "commands")]
use std::collections::HashMap;
#[cfg(feature = "commands")]
use serde::{Serialize, Deserialize};
#[cfg(feature = "commands")]
use playground_core_ecs::impl_component_data;
#[cfg(feature = "commands")]
use crate::types::ResourceId;
#[cfg(feature = "commands")]
use crate::commands::CommandBufferInfo;

/// Storage for command buffers and recording state
#[cfg(feature = "commands")]
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CommandBufferStorage {
    pub command_buffers: HashMap<ResourceId, CommandBufferInfo>,
    pub recording_buffer: Option<ResourceId>,
}

#[cfg(feature = "commands")]
impl_component_data!(CommandBufferStorage);