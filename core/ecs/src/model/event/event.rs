//! Event type

use bytes::Bytes;
use playground_modules_types::{ModelTrait, ModelId, ModelType, model_type_of};
use crate::model::{
    event::{EventId, Priority},
    entity::{EntityRef, EntityId, Generation},
};
use std::sync::Weak;

/// Event to be dispatched
#[derive(Clone, serde::Serialize, serde::Deserialize)]
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

impl ModelTrait for Event {
    fn model_id(&self) -> ModelId {
        self.id.0 as u64  // Convert EventId's u32 to u64 ModelId
    }

    fn model_type(&self) -> ModelType {
        model_type_of::<Event>()  // Runtime-generated, but deterministic
    }
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