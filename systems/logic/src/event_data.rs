use bytes::Bytes;
use serde::{Serialize, Deserialize};
use std::any::TypeId;
use std::collections::VecDeque;
use crate::entity::Entity;
use crate::error::{LogicResult, LogicError};

/// Concrete wrapper for event data that avoids Box<dyn Any>
#[derive(Clone)]
pub struct EventData {
    entity: Entity,
    data: Bytes,
    type_id: TypeId,
    priority: u8,
    persistent: bool,
}

impl EventData {
    /// Create new EventData from an event
    pub fn new<E: Serialize + 'static>(entity: Entity, event: E, priority: u8, persistent: bool) -> LogicResult<Self> {
        let data = bincode::serialize(&event)
            .map_err(|e| LogicError::SerializationError(e.to_string()))?
            .into();
        
        Ok(Self {
            entity,
            data,
            type_id: TypeId::of::<E>(),
            priority,
            persistent,
        })
    }
    
    /// Get the entity associated with this event
    pub fn entity(&self) -> Entity {
        self.entity
    }
    
    /// Get the type ID of this event
    pub fn type_id(&self) -> TypeId {
        self.type_id
    }
    
    /// Get the priority of this event
    pub fn priority(&self) -> u8 {
        self.priority
    }
    
    /// Check if this event is persistent
    pub fn is_persistent(&self) -> bool {
        self.persistent
    }
    
    /// Deserialize back to the original event type
    pub fn deserialize<E: for<'de> Deserialize<'de>>(&self) -> LogicResult<E> {
        bincode::deserialize(&self.data)
            .map_err(|e| LogicError::SerializationError(e.to_string()))
    }
}

/// Concrete event queue that avoids Box<dyn Any>
pub struct EventQueueData {
    queue_type: TypeId,
    events: VecDeque<EventData>,
    persistent: Vec<EventData>,
}

impl EventQueueData {
    /// Create a new event queue for a specific event type
    pub fn new(queue_type: TypeId) -> Self {
        Self {
            queue_type,
            events: VecDeque::new(),
            persistent: Vec::new(),
        }
    }
    
    /// Push an event to the queue
    pub fn push(&mut self, event: EventData) {
        if event.is_persistent() {
            self.persistent.push(event);
        } else {
            self.events.push_back(event);
        }
    }
    
    /// Drain all events from the queue
    pub fn drain(&mut self) -> Vec<EventData> {
        let mut result: Vec<_> = self.events.drain(..).collect();
        result.extend(self.persistent.iter().cloned());
        result
    }
    
    /// Clear persistent events
    pub fn clear_persistent(&mut self) {
        self.persistent.clear();
    }
    
    /// Get the type ID of events in this queue
    pub fn queue_type(&self) -> TypeId {
        self.queue_type
    }
    
    /// Check if the queue is empty
    pub fn is_empty(&self) -> bool {
        self.events.is_empty() && self.persistent.is_empty()
    }
    
    /// Get the total number of events
    pub fn len(&self) -> usize {
        self.events.len() + self.persistent.len()
    }
}