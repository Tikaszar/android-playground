//! World data structure

use std::collections::HashMap;
use std::sync::atomic::AtomicU32;
use playground_core_types::{Handle, handle, Shared, shared};
use crate::model::{
    entity::{EntityId, Generation},
    component::{Component, ComponentId},
    event::{EventId, Event, SubscriptionId},
    query::{QueryId, QueryFilter},
    storage::StorageId,
    system::SystemId,
};

/// The concrete World struct - data fields only, no logic!
pub struct World {
    /// Entity generation tracking
    pub entities: Shared<HashMap<EntityId, Generation>>,

    /// Component storage: entity -> component_id -> component
    pub components: Shared<HashMap<EntityId, HashMap<ComponentId, Component>>>,

    /// Next entity ID counter
    pub next_entity_id: AtomicU32,

    /// Event queue for deferred processing
    pub event_queue: Shared<Vec<Event>>,

    /// Pre-event handlers: event_id -> list of handler IDs
    pub pre_handlers: Shared<HashMap<EventId, Vec<SubscriptionId>>>,

    /// Post-event handlers: event_id -> list of handler IDs
    pub post_handlers: Shared<HashMap<EventId, Vec<SubscriptionId>>>,

    /// Next subscription ID
    pub next_subscription_id: AtomicU32,

    /// Query storage: query_id -> filter
    pub queries: Shared<HashMap<QueryId, QueryFilter>>,

    /// Next query ID counter
    pub next_query_id: AtomicU32,

    /// Storage metadata: storage_id -> (path, format)
    pub storages: Shared<HashMap<StorageId, (String, String)>>,

    /// Next storage ID counter
    pub next_storage_id: AtomicU32,

    /// System metadata: system_id -> (name, query_id, dependencies)
    pub systems: Shared<HashMap<SystemId, (String, QueryId, Vec<SystemId>)>>,

    /// Next system ID counter
    pub next_system_id: AtomicU32,
}

impl World {
    /// Create a new World instance - just data initialization, no logic!
    pub fn new() -> Handle<Self> {
        handle(Self {
            entities: shared(HashMap::new()),
            components: shared(HashMap::new()),
            next_entity_id: AtomicU32::new(1),
            event_queue: shared(Vec::new()),
            pre_handlers: shared(HashMap::new()),
            post_handlers: shared(HashMap::new()),
            next_subscription_id: AtomicU32::new(1),
            queries: shared(HashMap::new()),
            next_query_id: AtomicU32::new(1),
            storages: shared(HashMap::new()),
            next_storage_id: AtomicU32::new(1),
            systems: shared(HashMap::new()),
            next_system_id: AtomicU32::new(1),
        })
    }

    /// Create a weak reference to this World from a Handle
    pub fn downgrade(world: &Handle<Self>) -> super::WorldRef {
        super::WorldRef {
            world: std::sync::Arc::downgrade(world),
        }
    }
}