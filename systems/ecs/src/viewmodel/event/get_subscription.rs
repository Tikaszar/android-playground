//! Get subscription by ID

use playground_core_ecs::{World, SubscriptionId, Subscription, EcsResult, EcsError};

/// Get subscription by ID
pub async fn get_subscription(world: &World, subscription_id: SubscriptionId) -> EcsResult<Subscription> {
    // Get subscription
    let subscriptions = world.subscriptions.read().await;
    let subscription = subscriptions
        .get(&subscription_id)
        .cloned()
        .ok_or_else(|| EcsError::SubscriptionNotFound(subscription_id))?;

    Ok(subscription)
}
