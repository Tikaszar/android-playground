//! Subscribe to pre-events (can cancel the event)

use playground_core_ecs::{World, EventId, SubscriptionId, Subscription, Priority, EcsResult};
use std::sync::atomic::Ordering;

/// Subscribe to pre-events (can cancel the event)
pub async fn subscribe_pre(world: &World, event_id: EventId, handler_id: SubscriptionId) -> EcsResult<Subscription> {
    // Generate subscription ID
    let subscription_id = SubscriptionId::new(
        world.next_subscription_id.fetch_add(1, Ordering::SeqCst) as u64
    );

    // Add to pre-handlers
    let mut pre_handlers = world.pre_handlers.write().await;
    pre_handlers.entry(event_id).or_insert_with(Vec::new).push(subscription_id);
    drop(pre_handlers);

    // Store subscription
    let subscription = Subscription {
        id: subscription_id,
        event_id,
        priority: Priority::Pre,
        name: format!("pre_handler_{}", subscription_id.0),
    };

    let mut subscriptions = world.subscriptions.write().await;
    subscriptions.insert(subscription_id, subscription.clone());

    Ok(subscription)
}
