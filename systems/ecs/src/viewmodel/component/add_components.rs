//! Add multiple components to an entity in batch

use playground_modules_types::{ModuleResult, ModuleError};
use playground_core_ecs::{EntityId, Component};
use std::pin::Pin;
use std::future::Future;
use std::collections::HashMap;

#[derive(serde::Deserialize)]
struct AddComponentsArgs {
    entity_id: EntityId,
    components: Vec<Component>,
}

/// Add multiple components to an entity
pub fn add_components(args: &[u8]) -> Pin<Box<dyn Future<Output = ModuleResult<Vec<u8>>> + Send>> {
    let args = args.to_vec();
    Box::pin(async move {
        // Deserialize args
        let args: AddComponentsArgs = bincode::deserialize(&args)
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

        // Add all components
        {
            let mut components = world.components.write().await;
            let entity_components = components.entry(args.entity_id).or_insert_with(HashMap::new);
            for component in args.components {
                entity_components.insert(component.component_id, component);
            }
        }

        Ok(Vec::new())
    })
}
