//! Subscribe to post-events (notification after state change)

use playground_core_ecs::{World, EventId, SubscriptionId, Subscription, Priority, EcsResult};
use std::sync::atomic::Ordering;

/// Subscribe to post-events (notification after state change)
pub async fn subscribe_post(world: &World, event_id: EventId, handler_id: SubscriptionId) -> EcsResult<Subscription> {
    // Generate subscription ID
    let subscription_id = SubscriptionId::new(
        world.next_subscription_id.fetch_add(1, Ordering::SeqCst) as u64
    );

    // Add to post-handlers
    let mut post_handlers = world.post_handlers.write().await;
    post_handlers.entry(event_id).or_insert_with(Vec::new).push(subscription_id);
    drop(post_handlers);

    // Store subscription
    let subscription = Subscription {
        id: subscription_id,
        event_id,
        priority: Priority::Normal,
        name: format!("post_handler_{}", subscription_id.0),
    };

    let mut subscriptions = world.subscriptions.write().await;
    subscriptions.insert(subscription_id, subscription.clone());

    Ok(subscription)
}
