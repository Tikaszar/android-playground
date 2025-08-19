use crate::component::Component;
use crate::entity::Entity;
use tokio::sync::RwLock;
use std::any::TypeId;
use std::collections::VecDeque;
use std::sync::Arc;

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

/// Event queue for a specific event type
struct EventQueue<E: Event + Clone> {
    events: VecDeque<(Entity, E)>,
    persistent: Vec<(Entity, E)>,
}

impl<E: Event + Clone> EventQueue<E> {
    fn new() -> Self {
        Self {
            events: VecDeque::new(),
            persistent: Vec::new(),
        }
    }
    
    fn push(&mut self, entity: Entity, event: E) {
        if E::persistent() {
            self.persistent.push((entity, event));
        } else {
            self.events.push_back((entity, event));
        }
    }
    
    fn drain(&mut self) -> Vec<(Entity, E)> {
        let mut result: Vec<_> = self.events.drain(..).collect();
        result.extend(self.persistent.iter().map(|(e, evt)| (*e, evt.clone())));
        result
    }
    
    fn clear_persistent(&mut self) {
        self.persistent.clear();
    }
}

/// Event system manages all events in the ECS
pub struct EventSystem {
    queues: Arc<RwLock<fnv::FnvHashMap<TypeId, Box<dyn std::any::Any + Send + Sync>>>>,
    readers: Arc<RwLock<fnv::FnvHashMap<TypeId, Vec<EventReader>>>>,
}

use fnv::FnvHashMap;

impl EventSystem {
    pub fn new() -> Self {
        Self {
            queues: Arc::new(RwLock::new(FnvHashMap::default())),
            readers: Arc::new(RwLock::new(FnvHashMap::default())),
        }
    }
    
    /// Send an event to the system
    pub async fn send<E: Event + Clone + 'static>(&self, entity: Entity, event: E) {
        let mut queues = self.queues.write().await;
        let queue = queues
            .entry(TypeId::of::<E>())
            .or_insert_with(|| Box::new(EventQueue::<E>::new()));
        
        if let Some(typed_queue) = queue.downcast_mut::<EventQueue<E>>() {
            typed_queue.push(entity, event);
        }
    }
    
    /// Send multiple events in a batch
    pub async fn send_batch<E: Event + Clone + 'static>(&self, events: Vec<(Entity, E)>) {
        let mut queues = self.queues.write().await;
        let queue = queues
            .entry(TypeId::of::<E>())
            .or_insert_with(|| Box::new(EventQueue::<E>::new()));
        
        if let Some(typed_queue) = queue.downcast_mut::<EventQueue<E>>() {
            for (entity, event) in events {
                typed_queue.push(entity, event);
            }
        }
    }
    
    /// Create an event reader for a specific event type
    pub async fn reader<E: Event + 'static>(&self) -> EventReader {
        let reader = EventReader {
            event_type: TypeId::of::<E>(),
            last_read: Arc::new(RwLock::new(0)),
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
            // Clear non-persistent events
            // Real implementation would handle this per type
        }
    }
}

/// Reader for consuming events
#[derive(Clone)]
pub struct EventReader {
    event_type: TypeId,
    last_read: Arc<RwLock<usize>>,
}

impl EventReader {
    /// Read all new events since last read
    pub async fn read<E: Event + Clone + 'static>(&self, event_system: &EventSystem) -> Vec<(Entity, E)> {
        let queues = event_system.queues.read().await;
        
        if let Some(queue) = queues.get(&self.event_type) {
            if let Some(typed_queue) = queue.downcast_ref::<EventQueue<E>>() {
                // In real implementation, would track what's been read
                return typed_queue.events.iter().cloned().collect();
            }
        }
        
        Vec::new()
    }
    
    /// Check if there are unread events
    pub async fn has_events(&self, event_system: &EventSystem) -> bool {
        let queues = event_system.queues.read().await;
        
        if let Some(queue) = queues.get(&self.event_type) {
            // Check if queue has events
            return true; // Simplified
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