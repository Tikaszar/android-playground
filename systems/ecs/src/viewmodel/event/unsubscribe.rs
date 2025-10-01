//! Unsubscribe from an event

use playground_modules_types::{ModuleResult, ModuleError};
use playground_core_ecs::SubscriptionId;
use std::pin::Pin;
use std::future::Future;

#[derive(serde::Deserialize)]
struct UnsubscribeArgs {
    subscription_id: SubscriptionId,
}

/// Unsubscribe from an event
pub fn unsubscribe(args: &[u8]) -> Pin<Box<dyn Future<Output = ModuleResult<Vec<u8>>> + Send>> {
    let args = args.to_vec();
    Box::pin(async move {
        // Deserialize args
        let args: UnsubscribeArgs = bincode::deserialize(&args)
            .map_err(|e| ModuleError::DeserializationError(e.to_string()))?;

        // Get World
        let world = crate::state::get_world()
            .map_err(|e| ModuleError::Generic(e))?;

        // Remove from subscriptions
        {
            let mut subscriptions = world.subscriptions.write().await;
            subscriptions.remove(&args.subscription_id);
        }

        // Remove from pre-handlers
        {
            let mut pre_handlers = world.pre_handlers.write().await;
            for handlers in pre_handlers.values_mut() {
                handlers.retain(|id| *id != args.subscription_id);
            }
        }

        // Remove from post-handlers
        {
            let mut post_handlers = world.post_handlers.write().await;
            for handlers in post_handlers.values_mut() {
                handlers.retain(|id| *id != args.subscription_id);
            }
        }

        Ok(Vec::new())
    })
}
