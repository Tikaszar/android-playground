//! Subscribe to an event (generic subscription)

use playground_core_ecs::{World, EventId, Subscription, SubscriptionId, Priority, EcsResult};
use std::sync::atomic::Ordering;

/// Subscribe to an event (generic subscription)
pub async fn subscribe_event(world: &World, event_id: EventId, listener: String) -> EcsResult<Subscription> {
    // Generate subscription ID
    let subscription_id = SubscriptionId::new(
        world.next_subscription_id.fetch_add(1, Ordering::SeqCst) as u64
    );

    // Default to post-handler for generic subscriptions
    let mut post_handlers = world.post_handlers.write().await;
    post_handlers.entry(event_id).or_insert_with(Vec::new).push(subscription_id);
    drop(post_handlers);

    // Store subscription
    let subscription = Subscription {
        id: subscription_id,
        event_id,
        priority: Priority::Normal,
        name: listener,
    };

    let mut subscriptions = world.subscriptions.write().await;
    subscriptions.insert(subscription_id, subscription.clone());

    Ok(subscription)
}
