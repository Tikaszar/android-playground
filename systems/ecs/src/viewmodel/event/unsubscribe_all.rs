//! Unsubscribe all handlers from an event

use playground_modules_types::{ModuleResult, ModuleError};
use playground_core_ecs::EventId;
use std::pin::Pin;
use std::future::Future;

#[derive(serde::Deserialize)]
struct UnsubscribeAllArgs {
    event_id: EventId,
}

/// Unsubscribe all handlers from an event
/// Returns the number of subscriptions removed
pub fn unsubscribe_all(args: &[u8]) -> Pin<Box<dyn Future<Output = ModuleResult<Vec<u8>>> + Send>> {
    let args = args.to_vec();
    Box::pin(async move {
        // Deserialize args
        let args: UnsubscribeAllArgs = bincode::deserialize(&args)
            .map_err(|e| ModuleError::DeserializationError(e.to_string()))?;

        // Get World
        let world = crate::state::get_world()
            .map_err(|e| ModuleError::Generic(e))?;

        let mut removed_count = 0;

        // Get all subscription IDs for this event
        let mut subscription_ids = Vec::new();

        // From pre-handlers
        {
            let mut pre_handlers = world.pre_handlers.write().await;
            if let Some(handlers) = pre_handlers.remove(&args.event_id) {
                subscription_ids.extend(handlers);
            }
        }

        // From post-handlers
        {
            let mut post_handlers = world.post_handlers.write().await;
            if let Some(handlers) = post_handlers.remove(&args.event_id) {
                subscription_ids.extend(handlers);
            }
        }

        // Remove all subscriptions
        {
            let mut subscriptions = world.subscriptions.write().await;
            for sub_id in subscription_ids {
                if subscriptions.remove(&sub_id).is_some() {
                    removed_count += 1;
                }
            }
        }

        // Serialize result
        let result = bincode::serialize(&removed_count)
            .map_err(|e| ModuleError::SerializationError(e.to_string()))?;

        Ok(result)
    })
}
