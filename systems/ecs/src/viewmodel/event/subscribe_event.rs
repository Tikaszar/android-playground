//! Subscribe to events with a specific priority

use playground_modules_types::{ModuleResult, ModuleError};
use playground_core_ecs::{EventId, Priority, SubscriptionId};
use std::pin::Pin;
use std::future::Future;
use std::sync::atomic::Ordering;

#[derive(serde::Deserialize)]
struct SubscribeEventArgs {
    event_id: EventId,
    priority: Priority,
    name: String,
}

/// Subscribe to events with a specific priority
pub fn subscribe_event(args: &[u8]) -> Pin<Box<dyn Future<Output = ModuleResult<Vec<u8>>> + Send>> {
    Box::pin(async move {
        // Deserialize args
        let args: SubscribeEventArgs = bincode::deserialize(args)
            .map_err(|e| ModuleError::DeserializationError(e.to_string()))?;

        // Get World
        let world = crate::state::get_world()
            .map_err(|e| ModuleError::Generic(e))?;

        // Generate subscription ID
        let subscription_id = SubscriptionId(world.next_subscription_id.fetch_add(1, Ordering::SeqCst) as u64);

        // Add to appropriate handler map based on priority
        match args.priority {
            Priority::Pre => {
                let mut pre_handlers = world.pre_handlers.write().await;
                pre_handlers.entry(args.event_id).or_insert_with(Vec::new).push(subscription_id);
            }
            _ => {
                let mut post_handlers = world.post_handlers.write().await;
                post_handlers.entry(args.event_id).or_insert_with(Vec::new).push(subscription_id);
            }
        }

        // Serialize subscription ID
        let result = bincode::serialize(&subscription_id)
            .map_err(|e| ModuleError::SerializationError(e.to_string()))?;

        Ok(result)
    })
}