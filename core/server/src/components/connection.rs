//! Server connection component

use crate::types::*;
use playground_core_ecs::impl_component_data;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// A server connection as an ECS component
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConnection {
    pub id: ConnectionId,
    pub status: ConnectionStatus,
    pub established_at: u64,  // Timestamp in seconds since UNIX epoch
    pub last_activity: u64,   // Timestamp in seconds since UNIX epoch
    pub bytes_sent: u64,
    pub bytes_received: u64,
    pub messages_sent: u64,
    pub messages_received: u64,
    pub metadata: HashMap<String, String>,  // Generic metadata (IP, pipe name, etc.)
}

impl_component_data!(ServerConnection);

impl ServerConnection {
    pub fn new(id: ConnectionId) -> Self {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        Self {
            id,
            status: ConnectionStatus::Connecting,
            established_at: now,
            last_activity: now,
            bytes_sent: 0,
            bytes_received: 0,
            messages_sent: 0,
            messages_received: 0,
            metadata: HashMap::new(),
        }
    }
}