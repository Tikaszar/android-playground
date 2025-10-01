//! Event type

use bytes::Bytes;
use crate::model::{
    event::{EventId, Priority},
    entity::{EntityRef, EntityId, Generation},
};
use std::sync::Weak;

/// Event to be dispatched
#[derive(Clone)]
pub struct Event {
    /// Event identifier
    pub id: EventId,

    /// Event data (serialized)
    pub data: Bytes,

    /// Source entity reference
    pub source: EntityRef,

    /// Timestamp (milliseconds since epoch)
    pub timestamp: u64,

    /// Event priority
    pub priority: Priority,
}

impl Event {
    /// Create a new event
    pub fn new(id: EventId, data: Bytes) -> Self {
        Self {
            id,
            data,
            source: EntityRef {
                id: EntityId::null(),
                generation: Generation::invalid(),
                world: Weak::new(),
            },
            timestamp: 0, // Will be set by dispatcher
            priority: Priority::Normal,
        }
    }

    /// Set the source entity
    pub fn with_source(mut self, source: EntityRef) -> Self {
        self.source = source;
        self
    }

    /// Set the priority
    pub fn with_priority(mut self, priority: Priority) -> Self {
        self.priority = priority;
        self
    }
}