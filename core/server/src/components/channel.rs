//! Channel component for pub/sub messaging

use crate::types::*;
use playground_core_ecs::{EntityRef, impl_component_data};
use serde::{Deserialize, Serialize};

/// A message channel as an ECS component
#[cfg(feature = "channels")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerChannel {
    pub id: ChannelId,
    pub name: String,
    pub description: Option<String>,
    pub created_at: u64,  // Timestamp in seconds since UNIX epoch
    pub message_count: u64,
    #[serde(skip)]  // EntityRef can't be serialized
    pub subscribers: Vec<EntityRef>,  // References to connection entities
}

#[cfg(feature = "channels")]
impl_component_data!(ServerChannel);

#[cfg(feature = "channels")]
impl ServerChannel {
    pub fn new(id: ChannelId, name: String) -> Self {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        Self {
            id,
            name,
            description: None,
            created_at: now,
            message_count: 0,
            subscribers: Vec::new(),
        }
    }

    pub fn add_subscriber(&mut self, connection: EntityRef) {
        if !self.subscribers.iter().any(|s| s.id() == connection.id()) {
            self.subscribers.push(connection);
        }
    }

    pub fn remove_subscriber(&mut self, connection: &EntityRef) {
        self.subscribers.retain(|s| s.id() != connection.id());
    }
}