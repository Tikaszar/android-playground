//! Check if an event has any subscribers

use playground_modules_types::{ModuleResult, ModuleError};
use playground_core_ecs::EventId;
use std::pin::Pin;
use std::future::Future;

#[derive(serde::Deserialize)]
struct HasSubscribersArgs {
    event_id: EventId,
}

/// Check if an event has any subscribers
pub fn has_subscribers(args: &[u8]) -> Pin<Box<dyn Future<Output = ModuleResult<Vec<u8>>> + Send>> {
    let args = args.to_vec();
    Box::pin(async move {
        // Deserialize args
        let args: HasSubscribersArgs = bincode::deserialize(&args)
            .map_err(|e| ModuleError::DeserializationError(e.to_string()))?;

        // Get World
        let world = crate::state::get_world()
            .map_err(|e| ModuleError::Generic(e))?;

        // Check pre-handlers
        let has_pre = {
            let pre_handlers = world.pre_handlers.read().await;
            pre_handlers.get(&args.event_id).map_or(false, |v| !v.is_empty())
        };

        // Check post-handlers
        let has_post = {
            let post_handlers = world.post_handlers.read().await;
            post_handlers.get(&args.event_id).map_or(false, |v| !v.is_empty())
        };

        let has_any = has_pre || has_post;

        // Serialize result
        let result = bincode::serialize(&has_any)
            .map_err(|e| ModuleError::SerializationError(e.to_string()))?;

        Ok(result)
    })
}
