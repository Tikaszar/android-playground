//! World data structure

use std::collections::HashMap;
use playground_modules_types::{Handle, handle, Shared, shared, Atomic};
use crate::model::{
    entity::{EntityId, Generation},
    component::ComponentId,
    event::{EventId, Event, SubscriptionId, Subscription},
    query::{QueryId, QueryFilter},
    storage::StorageId,
    system::SystemId,
};

/// The concrete World struct - data fields only, no logic!
pub struct World {
    /// Entity generation tracking
    pub entities: Shared<HashMap<EntityId, Generation>>,

    /// Component registry: maps component_id to the system that owns it
    /// Components are stored in System.component_pools, not here
    pub component_registry: Shared<HashMap<ComponentId, SystemId>>,

    /// Next entity ID counter
    pub next_entity_id: Atomic<u64>,

    /// Event queue for deferred processing
    pub event_queue: Shared<Vec<Event>>,

    /// Pre-event handlers: event_id -> list of handler IDs
    pub pre_handlers: Shared<HashMap<EventId, Vec<SubscriptionId>>>,

    /// Post-event handlers: event_id -> list of handler IDs
    pub post_handlers: Shared<HashMap<EventId, Vec<SubscriptionId>>>,

    /// Next subscription ID
    pub next_subscription_id: Atomic<u64>,

    /// Subscription storage: subscription_id -> subscription details
    pub subscriptions: Shared<HashMap<SubscriptionId, Subscription>>,

    /// Query storage: query_id -> filter
    pub queries: Shared<HashMap<QueryId, QueryFilter>>,

    /// Next query ID counter
    pub next_query_id: Atomic<u64>,

    /// Storage metadata: storage_id -> (path, format)
    pub storages: Shared<HashMap<StorageId, (String, String)>>,

    /// Next storage ID counter
    pub next_storage_id: Atomic<u64>,

    /// System metadata: system_id -> (name, query_id, dependencies)
    pub systems: Shared<HashMap<SystemId, (String, QueryId, Vec<SystemId>)>>,

    /// Next system ID counter
    pub next_system_id: Atomic<u64>,
}

impl World {
    /// Create a new World instance - just data initialization, no logic!
    pub fn new() -> Handle<Self> {
        handle(Self {
            entities: shared(HashMap::new()),
            component_registry: shared(HashMap::new()),
            next_entity_id: Atomic::<u64>::new(1),
            event_queue: shared(Vec::new()),
            pre_handlers: shared(HashMap::new()),
            post_handlers: shared(HashMap::new()),
            next_subscription_id: Atomic::<u64>::new(1),
            subscriptions: shared(HashMap::new()),
            queries: shared(HashMap::new()),
            next_query_id: Atomic::<u64>::new(1),
            storages: shared(HashMap::new()),
            next_storage_id: Atomic::<u64>::new(1),
            systems: shared(HashMap::new()),
            next_system_id: Atomic::<u64>::new(1),
        })
    }

    /// Create a weak reference to this World from a Handle
    pub fn downgrade(world: &Handle<Self>) -> super::WorldRef {
        super::WorldRef {
            world: std::sync::Arc::downgrade(world),
        }
    }
}