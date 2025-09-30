//! World data structure

use std::collections::HashMap;
use std::sync::atomic::AtomicU32;
use playground_core_types::{Handle, handle, Shared, shared};
use crate::model::{
    entity::{EntityId, Generation},
    component::{Component, ComponentId},
    event::{EventId, Event, SubscriptionId},
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
        })
    }
}