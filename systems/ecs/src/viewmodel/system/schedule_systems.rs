//! Schedule systems based on dependencies

use playground_core_ecs::{World, System, SystemId, EcsResult};
use playground_modules_types::handle;
use std::collections::HashMap;

/// Schedule systems based on dependencies
/// Returns the execution order using topological sort
pub async fn schedule_systems(world: &World) -> EcsResult<Vec<System>> {
    let systems = world.systems.read().await;

    // Build dependency graph
    let mut graph: HashMap<SystemId, Vec<SystemId>> = HashMap::new();
    let mut in_degree: HashMap<SystemId, usize> = HashMap::new();

    for (system_id, (_, _, dependencies)) in systems.iter() {
        in_degree.entry(*system_id).or_insert(0);
        for dep in dependencies {
            graph.entry(*dep).or_insert_with(Vec::new).push(*system_id);
            *in_degree.entry(*system_id).or_insert(0) += 1;
        }
    }

    // Topological sort using Kahn's algorithm
    let mut queue: Vec<SystemId> = in_degree
        .iter()
        .filter(|(_, &degree)| degree == 0)
        .map(|(&id, _)| id)
        .collect();

    let mut sorted = Vec::new();

    while let Some(current) = queue.pop() {
        sorted.push(current);

        if let Some(dependents) = graph.get(&current) {
            for &dependent in dependents {
                if let Some(degree) = in_degree.get_mut(&dependent) {
                    *degree -= 1;
                    if *degree == 0 {
                        queue.push(dependent);
                    }
                }
            }
        }
    }

    // Check for cycles
    if sorted.len() != systems.len() {
        return Err(playground_core_ecs::EcsError::OperationFailed("System cyclic dependency detected".to_string()));
    }

    // Convert system IDs to System handles
    let mut result = Vec::new();
    for system_id in sorted {
        if let Some((name, query, dependencies)) = systems.get(&system_id) {
            result.push(System::new(
                system_id,
                name.clone(),
                *query,
                dependencies.clone(),
                handle(world.clone())
            ));
        }
    }

    Ok(result)
}
