//! Client state component

use crate::types::*;
use playground_core_ecs::impl_component_data;
use serde::{Deserialize, Serialize};

/// Client runtime state as an ECS component
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClientStateComponent {
    pub id: ClientId,
    pub state: ClientState,
    pub server_address: Option<String>,
    pub connected_at: Option<u64>,  // Timestamp in seconds since UNIX epoch
}

impl_component_data!(ClientStateComponent);

impl ClientStateComponent {
    pub fn new(id: ClientId) -> Self {
        Self {
            id,
            state: ClientState::Initializing,
            server_address: None,
            connected_at: None,
        }
    }

    pub fn connect(&mut self, address: String) {
        self.state = ClientState::Connecting;
        self.server_address = Some(address);
    }

    pub fn on_connected(&mut self) {
        self.state = ClientState::Connected;
        self.connected_at = Some(
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs()
        );
    }

    pub fn disconnect(&mut self) {
        self.state = ClientState::Disconnecting;
    }

    pub fn on_disconnected(&mut self) {
        self.state = ClientState::Disconnected;
        self.connected_at = None;
    }
}