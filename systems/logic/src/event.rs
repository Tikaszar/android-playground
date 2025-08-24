use crate::component::Component;
use crate::entity::Entity;
use crate::event_data::{EventData, EventQueueData};
use playground_core_types::{Handle, handle, Shared, shared};
use tokio::sync::RwLock;
use std::any::TypeId;
use std::collections::VecDeque;
use serde::Serialize;

/// Event trait - events are just components with special handling
pub trait Event: Component {
    /// Get event priority (higher = processed first)
    fn priority() -> u8 {
        128
    }
    
    /// Whether this event should be persisted across frames
    fn persistent() -> bool {
        false
    }
}

// EventQueue functionality is now in EventQueueData in event_data.rs

/// Event system manages all events in the ECS
pub struct EventSystem {
    queues: Shared<fnv::FnvHashMap<TypeId, EventQueueData>>,
    readers: Shared<fnv::FnvHashMap<TypeId, Vec<EventReader>>>,
}

use fnv::FnvHashMap;

impl EventSystem {
    pub fn new() -> Self {
        Self {
            queues: shared(FnvHashMap::default()),
            readers: shared(FnvHashMap::default()),
        }
    }
    
    /// Send an event to the system
    pub async fn send<E: Event + Clone + Serialize + 'static>(&self, entity: Entity, event: E) -> crate::error::LogicResult<()> {
        let type_id = TypeId::of::<E>();
        let event_data = EventData::new(entity, event, E::priority(), E::persistent())?;
        
        let mut queues = self.queues.write().await;
        let queue = queues
            .entry(type_id)
            .or_insert_with(|| EventQueueData::new(type_id));
        
        queue.push(event_data);
        Ok(())
    }
    
    /// Send multiple events in a batch
    pub async fn send_batch<E: Event + Clone + Serialize + 'static>(&self, events: Vec<(Entity, E)>) -> crate::error::LogicResult<()> {
        let type_id = TypeId::of::<E>();
        let mut queues = self.queues.write().await;
        let queue = queues
            .entry(type_id)
            .or_insert_with(|| EventQueueData::new(type_id));
        
        for (entity, event) in events {
            let event_data = EventData::new(entity, event, E::priority(), E::persistent())?;
            queue.push(event_data);
        }
        Ok(())
    }
    
    /// Create an event reader for a specific event type
    pub async fn reader<E: Event + 'static>(&self) -> EventReader {
        let reader = EventReader {
            event_type: TypeId::of::<E>(),
            last_read: shared(0),
        };
        
        self.readers
            .write().await
            .entry(TypeId::of::<E>())
            .or_insert_with(Vec::new)
            .push(reader.clone());
        
        reader
    }
    
    /// Process all events for this frame
    pub async fn process_events(&self) {
        let mut queues = self.queues.write().await;
        
        // Sort by priority
        let mut sorted_events: Vec<_> = queues.iter_mut().collect();
        sorted_events.sort_by_key(|(type_id, _)| {
            // Get priority from registered event types
            0u8 // Simplified - real implementation would track priorities
        });
        
        // Process each queue
        for (_type_id, queue) in sorted_events {
            // Events are processed by systems that read them
            // This just manages the queues
        }
    }
    
    /// Clear all non-persistent events
    pub async fn clear_frame_events(&self) {
        let mut queues = self.queues.write().await;
        for queue in queues.values_mut() {
            // Clear non-persistent events by draining the queue
            queue.drain();
        }
    }
}

/// Reader for consuming events
#[derive(Clone)]
pub struct EventReader {
    event_type: TypeId,
    last_read: Shared<usize>,
}

impl EventReader {
    /// Read all new events since last read
    pub async fn read<E: Event + Clone + for<'de> serde::Deserialize<'de> + 'static>(&self, event_system: &EventSystem) -> Vec<(Entity, E)> {
        let queues = event_system.queues.read().await;
        
        if let Some(queue) = queues.get(&self.event_type) {
            // Get all events and deserialize them
            let events = queue.drain();
            let mut result = Vec::new();
            for event_data in events {
                if let Ok(event) = event_data.deserialize::<E>() {
                    result.push((event_data.entity(), event));
                }
            }
            return result;
        }
        
        Vec::new()
    }
    
    /// Check if there are unread events
    pub async fn has_events(&self, event_system: &EventSystem) -> bool {
        let queues = event_system.queues.read().await;
        
        if let Some(queue) = queues.get(&self.event_type) {
            return !queue.is_empty();
        }
        
        false
    }
}

/// Common event types
#[derive(Clone, Debug)]
pub struct EntitySpawned {
    pub entity: Entity,
}

impl Component for EntitySpawned {
    fn type_name() -> &'static str {
        "EntitySpawned"
    }
}

impl Event for EntitySpawned {
    fn priority() -> u8 {
        200 // High priority
    }
}

#[derive(Clone, Debug)]
pub struct EntityDespawned {
    pub entity: Entity,
}

impl Component for EntityDespawned {
    fn type_name() -> &'static str {
        "EntityDespawned"
    }
}

impl Event for EntityDespawned {
    fn priority() -> u8 {
        200 // High priority
    }
}

#[derive(Clone, Debug)]
pub struct ComponentAdded {
    pub entity: Entity,
    pub component_type: TypeId,
}

impl Component for ComponentAdded {
    fn type_name() -> &'static str {
        "ComponentAdded"
    }
}

impl Event for ComponentAdded {
    fn priority() -> u8 {
        150
    }
}

#[derive(Clone, Debug)]
pub struct ComponentRemoved {
    pub entity: Entity,
    pub component_type: TypeId,
}

impl Component for ComponentRemoved {
    fn type_name() -> &'static str {
        "ComponentRemoved"
    }
}

impl Event for ComponentRemoved {
    fn priority() -> u8 {
        150
    }
}