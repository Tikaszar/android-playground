//! Replace a component on an entity (add or update)

use playground_modules_types::{ModuleResult, ModuleError};
use playground_core_ecs::{EntityId, Component};
use std::pin::Pin;
use std::future::Future;
use std::collections::HashMap;

#[derive(serde::Deserialize)]
struct ReplaceComponentArgs {
    entity_id: EntityId,
    component: Component,
}

/// Replace a component on an entity (add or update)
pub fn replace_component(args: &[u8]) -> Pin<Box<dyn Future<Output = ModuleResult<Vec<u8>>> + Send>> {
    let args = args.to_vec();
    Box::pin(async move {
        // Deserialize args
        let args: ReplaceComponentArgs = bincode::deserialize(&args)
            .map_err(|e| ModuleError::DeserializationError(e.to_string()))?;

        // Get World
        let world = crate::state::get_world()
            .map_err(|e| ModuleError::Generic(e))?;

        // Check entity exists
        let exists = {
            let entities = world.entities.read().await;
            entities.contains_key(&args.entity_id)
        };

        if !exists {
            return Err(ModuleError::Generic(format!("Entity {:?} not found", args.entity_id)));
        }

        // Replace component (insert overwrites existing)
        {
            let mut components = world.components.write().await;
            let entity_components = components.entry(args.entity_id).or_insert_with(HashMap::new);
            entity_components.insert(args.component.component_id, args.component);
        }

        Ok(Vec::new())
    })
}
