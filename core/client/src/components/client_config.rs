//! Client configuration component

use crate::types::*;
use playground_core_ecs::impl_component_data;
use serde::{Deserialize, Serialize};

/// Client configuration as an ECS component
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClientConfigComponent {
    pub config: ClientConfig,
}

impl_component_data!(ClientConfigComponent);

impl ClientConfigComponent {
    pub fn new(config: ClientConfig) -> Self {
        Self { config }
    }
}

impl Default for ClientConfigComponent {
    fn default() -> Self {
        Self {
            config: ClientConfig::default(),
        }
    }
}