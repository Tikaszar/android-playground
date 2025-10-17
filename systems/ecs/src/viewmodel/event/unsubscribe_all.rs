//! Unsubscribe all handlers from an event

use playground_core_ecs::{World, EventId, EcsResult};

/// Unsubscribe all handlers from an event
/// Returns the number of subscriptions removed
pub async fn unsubscribe_all(world: &World, event_id: EventId) -> EcsResult<usize> {
    let mut removed_count = 0;

    // Get all subscription IDs for this event
    let mut subscription_ids = Vec::new();

    // From pre-handlers
    let mut pre_handlers = world.pre_handlers.write().await;
    if let Some(handlers) = pre_handlers.remove(&event_id) {
        subscription_ids.extend(handlers);
    }
    drop(pre_handlers);

    // From post-handlers
    let mut post_handlers = world.post_handlers.write().await;
    if let Some(handlers) = post_handlers.remove(&event_id) {
        subscription_ids.extend(handlers);
    }
    drop(post_handlers);

    // Remove all subscriptions
    let mut subscriptions = world.subscriptions.write().await;
    for sub_id in subscription_ids {
        if subscriptions.remove(&sub_id).is_some() {
            removed_count += 1;
        }
    }

    Ok(removed_count)
}
