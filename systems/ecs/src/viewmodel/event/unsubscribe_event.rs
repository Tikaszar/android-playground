//! Unsubscribe from a specific event with listener

use playground_core_ecs::{World, EventId, EcsResult};

/// Unsubscribe from a specific event with listener
pub async fn unsubscribe_event(world: &World, event_id: EventId, listener: String) -> EcsResult<()> {
    // Find subscription ID matching the listener name
    let subscriptions = world.subscriptions.read().await;
    let subscription_id = subscriptions
        .iter()
        .find(|(_, sub)| sub.event_id == event_id && sub.name == listener)
        .map(|(id, _)| *id);
    drop(subscriptions);

    if let Some(sub_id) = subscription_id {
        // Remove from subscriptions
        let mut subscriptions = world.subscriptions.write().await;
        subscriptions.remove(&sub_id);
        drop(subscriptions);

        // Remove from pre-handlers
        let mut pre_handlers = world.pre_handlers.write().await;
        for handlers in pre_handlers.values_mut() {
            handlers.retain(|id| *id != sub_id);
        }
        drop(pre_handlers);

        // Remove from post-handlers
        let mut post_handlers = world.post_handlers.write().await;
        for handlers in post_handlers.values_mut() {
            handlers.retain(|id| *id != sub_id);
        }
    }

    Ok(())
}
