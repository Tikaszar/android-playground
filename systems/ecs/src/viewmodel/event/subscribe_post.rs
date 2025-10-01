//! Subscribe to post-events (notification after state change)

use playground_modules_types::{ModuleResult, ModuleError};
use playground_core_ecs::{EventId, SubscriptionId, Subscription, Priority};
use std::pin::Pin;
use std::future::Future;
use std::sync::atomic::Ordering;

#[derive(serde::Deserialize)]
struct SubscribePostArgs {
    event_id: EventId,
    handler_id: SubscriptionId,
}

#[derive(serde::Serialize)]
struct SubscribePostResult {
    id: SubscriptionId,
    event_id: EventId,
    priority: Priority,
    name: String,
}

/// Subscribe to post-events (notification after state change)
pub fn subscribe_post(args: &[u8]) -> Pin<Box<dyn Future<Output = ModuleResult<Vec<u8>>> + Send>> {
    let args = args.to_vec();
    Box::pin(async move {
        // Deserialize args
        let args: SubscribePostArgs = bincode::deserialize(&args)
            .map_err(|e| ModuleError::DeserializationError(e.to_string()))?;

        // Get World
        let world = crate::state::get_world()
            .map_err(|e| ModuleError::Generic(e))?;

        // Generate subscription ID
        let subscription_id = SubscriptionId::new(
            world.next_subscription_id.fetch_add(1, Ordering::SeqCst) as u64
        );

        // Add to post-handlers
        {
            let mut post_handlers = world.post_handlers.write().await;
            post_handlers.entry(args.event_id).or_insert_with(Vec::new).push(subscription_id);
        }

        // Store subscription
        let subscription = Subscription {
            id: subscription_id,
            event_id: args.event_id,
            priority: Priority::Normal,
            name: format!("post_handler_{}", subscription_id.0),
        };

        {
            let mut subscriptions = world.subscriptions.write().await;
            subscriptions.insert(subscription_id, subscription.clone());
        }

        // Serialize result
        let result_data = SubscribePostResult {
            id: subscription.id,
            event_id: subscription.event_id,
            priority: subscription.priority,
            name: subscription.name,
        };

        let result = bincode::serialize(&result_data)
            .map_err(|e| ModuleError::SerializationError(e.to_string()))?;

        Ok(result)
    })
}
