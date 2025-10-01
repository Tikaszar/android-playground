//! Unsubscribe from events

use playground_modules_types::{ModuleResult, ModuleError};
use playground_core_ecs::SubscriptionId;
use std::pin::Pin;
use std::future::Future;

/// Unsubscribe from events
pub fn unsubscribe_event(args: &[u8]) -> Pin<Box<dyn Future<Output = ModuleResult<Vec<u8>>> + Send>> {
    Box::pin(async move {
        // Deserialize subscription ID from args
        let subscription_id: SubscriptionId = bincode::deserialize(args)
            .map_err(|e| ModuleError::DeserializationError(e.to_string()))?;

        // Get World
        let world = crate::state::get_world()
            .map_err(|e| ModuleError::Generic(e))?;

        // Remove from pre-handlers
        {
            let mut pre_handlers = world.pre_handlers.write().await;
            for handlers in pre_handlers.values_mut() {
                handlers.retain(|id| *id != subscription_id);
            }
        }

        // Remove from post-handlers
        {
            let mut post_handlers = world.post_handlers.write().await;
            for handlers in post_handlers.values_mut() {
                handlers.retain(|id| *id != subscription_id);
            }
        }

        Ok(Vec::new())
    })
}