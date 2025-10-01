//! Event System API functions
//!
//! This module provides the View layer API contracts for event operations.
//! These functions are stubs that will be replaced by the actual implementations
//! from systems/ecs at compile time through conditional compilation.

use crate::{
    EcsResult, EcsError,
    model::{World, Event, EventId, Subscription, SubscriptionId},
};

/// Emit an event to be processed
pub async fn emit_event(_world: &World, _event: Event) -> EcsResult<()> {
    Err(EcsError::ModuleNotFound("emit_event not implemented - systems/ecs required".to_string()))
}

/// Emit multiple events in batch
pub async fn emit_batch(_world: &World, _events: Vec<Event>) -> EcsResult<()> {
    Err(EcsError::ModuleNotFound("emit_batch not implemented - systems/ecs required".to_string()))
}

/// Subscribe to pre-events (can cancel the event)
pub async fn subscribe_pre(
    _world: &World,
    _event_id: EventId,
    _handler_id: SubscriptionId,
) -> EcsResult<Subscription> {
    Err(EcsError::ModuleNotFound("subscribe_pre not implemented - systems/ecs required".to_string()))
}

/// Subscribe to post-events (notification after state change)
pub async fn subscribe_post(
    _world: &World,
    _event_id: EventId,
    _handler_id: SubscriptionId,
) -> EcsResult<Subscription> {
    Err(EcsError::ModuleNotFound("subscribe_post not implemented - systems/ecs required".to_string()))
}

/// Unsubscribe from an event
pub async fn unsubscribe(_world: &World, _subscription: Subscription) -> EcsResult<()> {
    Err(EcsError::ModuleNotFound("unsubscribe not implemented - systems/ecs required".to_string()))
}

/// Unsubscribe all handlers from an event
/// Returns the number of subscriptions removed
pub async fn unsubscribe_all(_world: &World, _event_id: EventId) -> EcsResult<usize> {
    Err(EcsError::ModuleNotFound("unsubscribe_all not implemented - systems/ecs required".to_string()))
}

/// Process the event queue, dispatching to handlers
/// Returns the number of events processed
pub async fn process_event_queue(_world: &World) -> EcsResult<usize> {
    Err(EcsError::ModuleNotFound("process_event_queue not implemented - systems/ecs required".to_string()))
}

/// Clear the event queue without processing
pub async fn clear_event_queue(_world: &World) -> EcsResult<()> {
    Err(EcsError::ModuleNotFound("clear_event_queue not implemented - systems/ecs required".to_string()))
}

/// Get pending events without processing them
pub async fn get_pending_events(_world: &World) -> EcsResult<Vec<Event>> {
    Err(EcsError::ModuleNotFound("get_pending_events not implemented - systems/ecs required".to_string()))
}

/// Check if an event has any subscribers
pub async fn has_subscribers(_world: &World, _event_id: EventId) -> EcsResult<bool> {
    Err(EcsError::ModuleNotFound("has_subscribers not implemented - systems/ecs required".to_string()))
}

/// Get all subscriptions for an event
pub async fn get_subscriptions(_world: &World, _event_id: EventId) -> EcsResult<Vec<Subscription>> {
    Err(EcsError::ModuleNotFound("get_subscriptions not implemented - systems/ecs required".to_string()))
}

/// Get subscription by ID
pub async fn get_subscription(_world: &World, _subscription_id: SubscriptionId) -> EcsResult<Subscription> {
    Err(EcsError::ModuleNotFound("get_subscription not implemented - systems/ecs required".to_string()))
}

/// Emit an event with specific priority
pub async fn emit_event_with_priority(_world: &World, _event: Event, _priority: crate::model::event::Priority) -> EcsResult<()> {
    Err(EcsError::ModuleNotFound("emit_event_with_priority not implemented - systems/ecs required".to_string()))
}

/// Process only high priority events
pub async fn process_high_priority_events(_world: &World) -> EcsResult<usize> {
    Err(EcsError::ModuleNotFound("process_high_priority_events not implemented - systems/ecs required".to_string()))
}

/// Get the queue size
pub async fn get_event_queue_size(_world: &World) -> EcsResult<usize> {
    Err(EcsError::ModuleNotFound("get_event_queue_size not implemented - systems/ecs required".to_string()))
}