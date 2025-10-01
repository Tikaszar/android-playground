//! Add a component to an entity

use playground_modules_types::{ModuleResult, ModuleError};
use playground_core_ecs::{EntityId, Component};
use std::pin::Pin;
use std::future::Future;
use std::collections::HashMap;

#[derive(serde::Deserialize)]
struct AddComponentArgs {
    entity_id: EntityId,
    component: Component,
}

/// Add a component to an entity
pub fn add_component(args: &[u8]) -> Pin<Box<dyn Future<Output = ModuleResult<Vec<u8>>> + Send>> {
    Box::pin(async move {
        // Deserialize args
        let args: AddComponentArgs = bincode::deserialize(args)
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

        // Add component
        {
            let mut components = world.components.write().await;
            let entity_components = components.entry(args.entity_id).or_insert_with(HashMap::new);
            entity_components.insert(args.component.component_id, args.component);
        }

        Ok(Vec::new())
    })
}