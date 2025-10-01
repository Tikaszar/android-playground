//! Get all subscriptions for an event

use playground_modules_types::{ModuleResult, ModuleError};
use playground_core_ecs::{EventId, Subscription};
use std::pin::Pin;
use std::future::Future;

#[derive(serde::Deserialize)]
struct GetSubscriptionsArgs {
    event_id: EventId,
}

/// Get all subscriptions for an event
pub fn get_subscriptions(args: &[u8]) -> Pin<Box<dyn Future<Output = ModuleResult<Vec<u8>>> + Send>> {
    let args = args.to_vec();
    Box::pin(async move {
        // Deserialize args
        let args: GetSubscriptionsArgs = bincode::deserialize(&args)
            .map_err(|e| ModuleError::DeserializationError(e.to_string()))?;

        // Get World
        let world = crate::state::get_world()
            .map_err(|e| ModuleError::Generic(e))?;

        // Collect all subscription IDs for this event
        let mut subscription_ids = Vec::new();

        // From pre-handlers
        {
            let pre_handlers = world.pre_handlers.read().await;
            if let Some(handlers) = pre_handlers.get(&args.event_id) {
                subscription_ids.extend(handlers.clone());
            }
        }

        // From post-handlers
        {
            let post_handlers = world.post_handlers.read().await;
            if let Some(handlers) = post_handlers.get(&args.event_id) {
                subscription_ids.extend(handlers.clone());
            }
        }

        // Get subscription details
        let subscriptions: Vec<Subscription> = {
            let subscriptions = world.subscriptions.read().await;
            subscription_ids
                .iter()
                .filter_map(|id| subscriptions.get(id).cloned())
                .collect()
        };

        // Serialize result
        let result = bincode::serialize(&subscriptions)
            .map_err(|e| ModuleError::SerializationError(e.to_string()))?;

        Ok(result)
    })
}
