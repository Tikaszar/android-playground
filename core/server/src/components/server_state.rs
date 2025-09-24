//! Server state component

use playground_core_ecs::impl_component_data;
use serde::{Deserialize, Serialize};

/// Server runtime state as an ECS component
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerState {
    pub is_running: bool,
    pub started_at: Option<u64>,  // Timestamp in seconds since UNIX epoch
}

impl_component_data!(ServerState);

impl ServerState {
    pub fn new() -> Self {
        Self {
            is_running: false,
            started_at: None,
        }
    }

    pub fn start(&mut self) {
        self.is_running = true;
        self.started_at = Some(
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs()
        );
    }

    pub fn stop(&mut self) {
        self.is_running = false;
    }
}

impl Default for ServerState {
    fn default() -> Self {
        Self::new()
    }
}