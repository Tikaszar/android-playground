//! Event System API functions

use bytes::Bytes;
use crate::{
    EcsResult,
    model::{World, EventId, Priority, SubscriptionId},
};

/// Publish a pre-event that can be cancelled
/// Returns false if any handler cancelled the event
pub async fn publish_pre_event(
    _world: &World,
    _event_id: EventId,
    _data: Bytes,
) -> EcsResult<bool> {
    todo!("Implemented by systems/ecs")
}

/// Publish a post-event (notification after state change)
pub async fn publish_post_event(
    _world: &World,
    _event_id: EventId,
    _data: Bytes,
) -> EcsResult<()> {
    todo!("Implemented by systems/ecs")
}

/// Subscribe to events with a specific priority
pub async fn subscribe_event(
    _world: &World,
    _event_id: EventId,
    _priority: Priority,
    _name: String,
) -> EcsResult<SubscriptionId> {
    todo!("Implemented by systems/ecs")
}

/// Unsubscribe from events
pub async fn unsubscribe_event(
    _world: &World,
    _subscription_id: SubscriptionId,
) -> EcsResult<()> {
    todo!("Implemented by systems/ecs")
}