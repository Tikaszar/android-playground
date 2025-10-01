//! Event subscription types

use crate::model::event::{EventId, Priority};

/// Subscription ID for tracking
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub struct SubscriptionId(pub u64);

impl SubscriptionId {
    /// Create a new subscription ID
    pub fn new(id: u64) -> Self {
        Self(id)
    }
}

/// Event subscription information
pub struct Subscription {
    /// Unique subscription ID
    pub id: SubscriptionId,

    /// Event to subscribe to
    pub event_id: EventId,

    /// Handler priority
    pub priority: Priority,

    /// Handler name (for debugging)
    pub name: String,
}