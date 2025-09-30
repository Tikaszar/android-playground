//! Event type

use bytes::Bytes;
use crate::model::{
    event::{EventId, Priority},
    entity::Entity,
};

/// Event to be dispatched
#[derive(Clone)]
pub struct Event {
    /// Event identifier
    pub id: EventId,

    /// Event data (serialized)
    pub data: Bytes,

    /// Optional source entity
    pub source: Option<Entity>,

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
            source: None,
            timestamp: 0, // Will be set by dispatcher
            priority: Priority::Normal,
        }
    }

    /// Set the source entity
    pub fn with_source(mut self, entity: Entity) -> Self {
        self.source = Some(entity);
        self
    }

    /// Set the priority
    pub fn with_priority(mut self, priority: Priority) -> Self {
        self.priority = priority;
        self
    }
}