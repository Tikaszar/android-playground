//! Event view stub implementation

use async_trait::async_trait;
use playground_modules_types::{ViewFragmentTrait, ViewId, FragmentId};
use crate::{
    EcsResult, EcsError,
    model::{World, Event, EventId, Priority, Subscription, SubscriptionId},
    view::event::EventView,
};

pub const EVENT_FRAGMENT_ID: FragmentId = 0x0003;

/// EcsView implementation for EventView fragment
pub struct EcsView;

#[async_trait]
impl ViewFragmentTrait for EcsView {
    fn view_id(&self) -> ViewId {
        crate::ECS_VIEW_ID
    }

    fn fragment_id(&self) -> FragmentId {
        EVENT_FRAGMENT_ID
    }
}

#[async_trait]
impl EventView for EcsView {
    async fn emit_event(&self, _world: &World, _event: Event) -> EcsResult<()> {
        Err(EcsError::NotImplemented("ViewModel not bound".into()))
    }

    async fn emit_batch(&self, _world: &World, _events: Vec<Event>) -> EcsResult<()> {
        Err(EcsError::NotImplemented("ViewModel not bound".into()))
    }

    async fn subscribe_pre(&self, _world: &World, _event_id: EventId, _handler_id: SubscriptionId) -> EcsResult<Subscription> {
        Err(EcsError::NotImplemented("ViewModel not bound".into()))
    }

    async fn subscribe_post(&self, _world: &World, _event_id: EventId, _handler_id: SubscriptionId) -> EcsResult<Subscription> {
        Err(EcsError::NotImplemented("ViewModel not bound".into()))
    }

    async fn unsubscribe(&self, _world: &World, _subscription: Subscription) -> EcsResult<()> {
        Err(EcsError::NotImplemented("ViewModel not bound".into()))
    }

    async fn unsubscribe_all(&self, _world: &World, _event_id: EventId) -> EcsResult<usize> {
        Err(EcsError::NotImplemented("ViewModel not bound".into()))
    }

    async fn process_event_queue(&self, _world: &World) -> EcsResult<usize> {
        Err(EcsError::NotImplemented("ViewModel not bound".into()))
    }

    async fn clear_event_queue(&self, _world: &World) -> EcsResult<()> {
        Err(EcsError::NotImplemented("ViewModel not bound".into()))
    }

    async fn get_pending_events(&self, _world: &World) -> EcsResult<Vec<Event>> {
        Err(EcsError::NotImplemented("ViewModel not bound".into()))
    }

    async fn has_subscribers(&self, _world: &World, _event_id: EventId) -> EcsResult<bool> {
        Err(EcsError::NotImplemented("ViewModel not bound".into()))
    }

    async fn get_subscriptions(&self, _world: &World, _event_id: EventId) -> EcsResult<Vec<Subscription>> {
        Err(EcsError::NotImplemented("ViewModel not bound".into()))
    }

    async fn get_subscription(&self, _world: &World, _subscription_id: SubscriptionId) -> EcsResult<Subscription> {
        Err(EcsError::NotImplemented("ViewModel not bound".into()))
    }

    async fn emit_event_with_priority(&self, _world: &World, _event: Event, _priority: Priority) -> EcsResult<()> {
        Err(EcsError::NotImplemented("ViewModel not bound".into()))
    }

    async fn process_high_priority_events(&self, _world: &World) -> EcsResult<usize> {
        Err(EcsError::NotImplemented("ViewModel not bound".into()))
    }

    async fn get_event_queue_size(&self, _world: &World) -> EcsResult<usize> {
        Err(EcsError::NotImplemented("ViewModel not bound".into()))
    }
}