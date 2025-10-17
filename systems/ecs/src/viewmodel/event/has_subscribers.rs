//! Check if an event has any subscribers

use playground_core_ecs::{World, EventId, EcsResult};

/// Check if an event has any subscribers
pub async fn has_subscribers(world: &World, event_id: EventId) -> EcsResult<bool> {
    // Check pre-handlers
    let pre_handlers = world.pre_handlers.read().await;
    let has_pre = pre_handlers.get(&event_id).map_or(false, |v| !v.is_empty());
    drop(pre_handlers);

    // Check post-handlers
    let post_handlers = world.post_handlers.read().await;
    let has_post = post_handlers.get(&event_id).map_or(false, |v| !v.is_empty());

    let has_any = has_pre || has_post;

    Ok(has_any)
}
