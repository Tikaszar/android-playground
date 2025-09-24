//! Network state management for ECS entities
//!
//! This module manages references to ECS entities for servers and clients,
//! along with the actual network implementation details.

use std::collections::HashMap;
use playground_core_types::{Handle, handle, Shared, shared};
use playground_core_ecs::Entity;
use playground_core_server::ConnectionId;
use once_cell::sync::Lazy;

use crate::server::NetworkServer;

/// Global network state using Lazy initialization
pub static NETWORK_STATE: Lazy<NetworkState> = Lazy::new(|| NetworkState {
    server_entity: shared(None),
    server_impl: shared(None),
    client_entity: shared(None),
    connection_entities: shared(HashMap::new()),
    connection_senders: shared(HashMap::new()),
});

/// Network state that bridges ECS entities with network implementation
pub struct NetworkState {
    /// The server entity in the ECS
    pub server_entity: Shared<Option<Entity>>,

    /// The actual network server implementation (internal detail)
    pub server_impl: Shared<Option<Handle<NetworkServer>>>,

    /// The client entity in the ECS
    pub client_entity: Shared<Option<Entity>>,

    /// Map of connection IDs to their entities
    pub connection_entities: Shared<HashMap<ConnectionId, Entity>>,

    /// Map of connection IDs to their WebSocket senders (implementation detail)
    pub connection_senders: Shared<HashMap<ConnectionId, tokio::sync::mpsc::Sender<Vec<u8>>>>,
}

impl NetworkState {
    /// Initialize the network state (called during system registration)
    pub async fn initialize() {
        // Force lazy initialization by accessing NETWORK_STATE
        let _ = &*NETWORK_STATE;
    }
}