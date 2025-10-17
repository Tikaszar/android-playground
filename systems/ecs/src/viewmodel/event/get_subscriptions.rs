//! Get all subscriptions for an event

use playground_core_ecs::{World, EventId, Subscription, EcsResult};

/// Get all subscriptions for an event
pub async fn get_subscriptions(world: &World, event_id: EventId) -> EcsResult<Vec<Subscription>> {
    // Collect all subscription IDs for this event
    let mut subscription_ids = Vec::new();

    // From pre-handlers
    let pre_handlers = world.pre_handlers.read().await;
    if let Some(handlers) = pre_handlers.get(&event_id) {
        subscription_ids.extend(handlers.clone());
    }
    drop(pre_handlers);

    // From post-handlers
    let post_handlers = world.post_handlers.read().await;
    if let Some(handlers) = post_handlers.get(&event_id) {
        subscription_ids.extend(handlers.clone());
    }
    drop(post_handlers);

    // Get subscription details
    let subscriptions_map = world.subscriptions.read().await;
    let subscriptions: Vec<Subscription> = subscription_ids
        .iter()
        .filter_map(|id| subscriptions_map.get(id).cloned())
        .collect();

    Ok(subscriptions)
}
