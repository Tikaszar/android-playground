//! Get all entities that have a specific component

use playground_core_ecs::{World, Entity, ComponentId, EcsResult};

/// Get all entities that have a specific component
pub async fn get_entities_with_component(world: &World, component_id: ComponentId) -> EcsResult<Vec<Entity>> {
    // Find all entities with the component
    let entities: Vec<Entity> = {
        let components = world.components.read().await;
        let entity_gens = world.entities.read().await;

        components
            .iter()
            .filter(|(_, entity_components)| entity_components.contains_key(&component_id))
            .filter_map(|(entity_id, _)| {
                entity_gens.get(entity_id).map(|generation| Entity { id: *entity_id, generation: *generation })
            })
            .collect()
    };

    Ok(entities)
}
