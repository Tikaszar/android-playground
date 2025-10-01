//! Get subscription by ID

use playground_modules_types::{ModuleResult, ModuleError};
use playground_core_ecs::SubscriptionId;
use std::pin::Pin;
use std::future::Future;

#[derive(serde::Deserialize)]
struct GetSubscriptionArgs {
    subscription_id: SubscriptionId,
}

/// Get subscription by ID
pub fn get_subscription(args: &[u8]) -> Pin<Box<dyn Future<Output = ModuleResult<Vec<u8>>> + Send>> {
    let args = args.to_vec();
    Box::pin(async move {
        // Deserialize args
        let args: GetSubscriptionArgs = bincode::deserialize(&args)
            .map_err(|e| ModuleError::DeserializationError(e.to_string()))?;

        // Get World
        let world = crate::state::get_world()
            .map_err(|e| ModuleError::Generic(e))?;

        // Get subscription
        let subscription = {
            let subscriptions = world.subscriptions.read().await;
            subscriptions.get(&args.subscription_id).cloned()
                .ok_or_else(|| ModuleError::Generic(format!("Subscription not found: {:?}", args.subscription_id)))?
        };

        // Serialize result
        let result = bincode::serialize(&subscription)
            .map_err(|e| ModuleError::SerializationError(e.to_string()))?;

        Ok(result)
    })
}
