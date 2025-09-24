//! Server configuration component

use crate::types::*;
use playground_core_ecs::impl_component_data;
use serde::{Deserialize, Serialize};

/// Server configuration as an ECS component
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfigComponent {
    pub config: ServerConfig,
}

impl_component_data!(ServerConfigComponent);

impl ServerConfigComponent {
    pub fn new(config: ServerConfig) -> Self {
        Self { config }
    }
}

impl Default for ServerConfigComponent {
    fn default() -> Self {
        Self {
            config: ServerConfig::default(),
        }
    }
}