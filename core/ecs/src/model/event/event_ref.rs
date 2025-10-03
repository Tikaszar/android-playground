//! Event weak reference

use std::sync::Weak;
use playground_modules_types::Handle;
use crate::model::{
    event::EventId,
    world::World,
};

/// A weak reference to an event in the queue
///
/// This is safe to pass around without keeping events alive.
/// Follows the same pattern as EntityRef and ComponentRef.
#[derive(Clone, Debug)]
pub struct EventRef {
    pub event_id: EventId,
    pub timestamp: u64,
    pub world: Weak<World>,
}

impl EventRef {
    /// Create a new event reference
    pub fn new(event_id: EventId, timestamp: u64, world: Weak<World>) -> Self {
        Self {
            event_id,
            timestamp,
            world,
        }
    }

    /// Get the event ID
    pub fn event_id(&self) -> EventId {
        self.event_id
    }

    /// Get the timestamp
    pub fn timestamp(&self) -> u64 {
        self.timestamp
    }

    /// Check if the weak reference is still valid (world exists)
    pub fn is_valid(&self) -> bool {
        self.world.upgrade().is_some()
    }

    /// Try to get a strong reference to the world
    pub fn world(&self) -> Option<Handle<World>> {
        self.world.upgrade().map(Handle::from)
    }
}

impl PartialEq for EventRef {
    fn eq(&self, other: &Self) -> bool {
        self.event_id == other.event_id && self.timestamp == other.timestamp
    }
}

impl Eq for EventRef {}

/// Serialization support for EventRef
impl serde::Serialize for EventRef {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut state = serializer.serialize_struct("EventRef", 2)?;
        state.serialize_field("event_id", &self.event_id)?;
        state.serialize_field("timestamp", &self.timestamp)?;
        state.end()
    }
}

/// Deserialization creates an invalid reference (needs to be fixed up)
impl<'de> serde::Deserialize<'de> for EventRef {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(serde::Deserialize)]
        struct EventRefData {
            event_id: EventId,
            timestamp: u64,
        }

        let data = EventRefData::deserialize(deserializer)?;
        Ok(EventRef {
            event_id: data.event_id,
            timestamp: data.timestamp,
            world: Weak::new(), // Invalid weak reference - needs to be fixed up
        })
    }
}
