//! Event view trait - API contract only

use async_trait::async_trait;
use crate::{
    EcsResult,
    model::{World, Event, EventId, Priority, Subscription, SubscriptionId},
};

/// Event system API contract
#[async_trait]
pub trait EventView: Send + Sync {
    /// Emit an event to be processed
    async fn emit_event(&self, world: &World, event: Event) -> EcsResult<()>;

    /// Emit multiple events in batch
    async fn emit_batch(&self, world: &World, events: Vec<Event>) -> EcsResult<()>;

    /// Subscribe to pre-events (can cancel the event)
    async fn subscribe_pre(&self, world: &World, event_id: EventId, handler_id: SubscriptionId) -> EcsResult<Subscription>;

    /// Subscribe to post-events (notification after state change)
    async fn subscribe_post(&self, world: &World, event_id: EventId, handler_id: SubscriptionId) -> EcsResult<Subscription>;

    /// Unsubscribe from an event
    async fn unsubscribe(&self, world: &World, subscription: Subscription) -> EcsResult<()>;

    /// Unsubscribe all handlers from an event
    /// Returns the number of subscriptions removed
    async fn unsubscribe_all(&self, world: &World, event_id: EventId) -> EcsResult<usize>;

    /// Process the event queue, dispatching to handlers
    /// Returns the number of events processed
    async fn process_event_queue(&self, world: &World) -> EcsResult<usize>;

    /// Clear the event queue without processing
    async fn clear_event_queue(&self, world: &World) -> EcsResult<()>;

    /// Get pending events without processing them
    async fn get_pending_events(&self, world: &World) -> EcsResult<Vec<Event>>;

    /// Check if an event has any subscribers
    async fn has_subscribers(&self, world: &World, event_id: EventId) -> EcsResult<bool>;

    /// Get all subscriptions for an event
    async fn get_subscriptions(&self, world: &World, event_id: EventId) -> EcsResult<Vec<Subscription>>;

    /// Get subscription by ID
    async fn get_subscription(&self, world: &World, subscription_id: SubscriptionId) -> EcsResult<Subscription>;

    /// Emit an event with specific priority
    async fn emit_event_with_priority(&self, world: &World, event: Event, priority: Priority) -> EcsResult<()>;

    /// Process only high priority events
    async fn process_high_priority_events(&self, world: &World) -> EcsResult<usize>;

    /// Get the queue size
    async fn get_event_queue_size(&self, world: &World) -> EcsResult<usize>;
}