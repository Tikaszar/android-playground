//! Unsubscribe from an event

use playground_core_ecs::{World, Subscription, EcsResult};

/// Unsubscribe from an event
pub async fn unsubscribe(world: &World, subscription: Subscription) -> EcsResult<()> {
    // Remove from subscriptions
    let mut subscriptions = world.subscriptions.write().await;
    subscriptions.remove(&subscription.id);
    drop(subscriptions);

    // Remove from pre-handlers
    let mut pre_handlers = world.pre_handlers.write().await;
    for handlers in pre_handlers.values_mut() {
        handlers.retain(|id| *id != subscription.id);
    }
    drop(pre_handlers);

    // Remove from post-handlers
    let mut post_handlers = world.post_handlers.write().await;
    for handlers in post_handlers.values_mut() {
        handlers.retain(|id| *id != subscription.id);
    }

    Ok(())
}
