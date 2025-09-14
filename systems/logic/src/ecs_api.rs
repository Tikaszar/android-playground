//! ECS API module for systems/logic
//! 
//! This provides the public API for all ECS operations.
//! All functions forward to the World command processor in systems/ecs.

use playground_core_ecs::{
    EntityId, ComponentId, ChannelId, StorageType,
    EcsResult, ComponentData,
};
use bytes::Bytes;

/// Spawn a single entity without components
pub async fn spawn_entity() -> EcsResult<EntityId> {
    playground_core_ecs::spawn_entity().await
}

/// Spawn multiple entities
pub async fn spawn_entities(count: usize) -> EcsResult<Vec<EntityId>> {
    playground_core_ecs::spawn_entities(count).await
}

/// Despawn an entity and all its components
pub async fn despawn_entity(entity: EntityId) -> EcsResult<()> {
    playground_core_ecs::despawn_entity(entity).await
}

/// Despawn multiple entities
pub async fn despawn_entities(entities: Vec<EntityId>) -> EcsResult<()> {
    playground_core_ecs::despawn_entities(entities).await
}

/// Add a component to an entity
pub async fn add_component<T: ComponentData>(entity: EntityId, component: T) -> EcsResult<()> {
    playground_core_ecs::add_component(entity, component).await
}

/// Add a component from raw bytes
pub async fn add_component_bytes(entity: EntityId, component_id: ComponentId, data: Bytes) -> EcsResult<()> {
    playground_core_ecs::add_component_bytes(entity, component_id, data).await
}

/// Remove a component from an entity
pub async fn remove_component(entity: EntityId, component_id: ComponentId) -> EcsResult<()> {
    playground_core_ecs::remove_component(entity, component_id).await
}

/// Get a component from an entity
pub async fn get_component<T: ComponentData>(entity: EntityId) -> EcsResult<T> {
    playground_core_ecs::get_component(entity).await
}

/// Get a component as raw bytes
pub async fn get_component_bytes(entity: EntityId, component_id: ComponentId) -> EcsResult<Bytes> {
    playground_core_ecs::get_component_bytes(entity, component_id).await
}

/// Check if an entity has a component
pub async fn has_component(entity: EntityId, component_id: ComponentId) -> bool {
    playground_core_ecs::has_component(entity, component_id).await
}

/// Check if an entity is alive
pub async fn is_entity_alive(entity: EntityId) -> bool {
    playground_core_ecs::is_alive(entity).await
}

/// Register a component type with the World
pub async fn register_component<T: ComponentData>() -> EcsResult<()> {
    playground_core_ecs::register_component::<T>().await
}

/// Register a component type with specific storage
pub async fn register_component_with_storage<T: ComponentData>(storage_type: StorageType) -> EcsResult<()> {
    playground_core_ecs::register_component_with_storage::<T>(storage_type).await
}

/// Query for entities with specific components
pub async fn query_entities(required: Vec<ComponentId>, excluded: Vec<ComponentId>) -> EcsResult<Vec<EntityId>> {
    playground_core_ecs::query_entities(required, excluded).await
}

/// Update the World (execute all systems)
pub async fn update_world(delta_time: f32) -> EcsResult<()> {
    playground_core_ecs::update_world(delta_time).await
}

/// Publish a message to a channel
pub async fn publish_message(channel: ChannelId, message: Bytes) -> EcsResult<()> {
    playground_core_ecs::publish(channel, message).await
}

/// Subscribe to a channel
pub async fn subscribe_to_channel(channel: ChannelId, handler_id: String) -> EcsResult<()> {
    playground_core_ecs::subscribe(channel, handler_id).await
}

/// Unsubscribe from a channel
pub async fn unsubscribe_from_channel(channel: ChannelId, handler_id: &str) -> EcsResult<()> {
    playground_core_ecs::unsubscribe(channel, handler_id).await
}

/// Run garbage collection
pub async fn run_gc() -> EcsResult<usize> {
    playground_core_ecs::run_gc().await
}

/// Check if the World is empty
pub async fn is_world_empty() -> bool {
    playground_core_ecs::is_empty().await
}

/// Send a command to a system
pub async fn send_to_system(target_system: &str, command_type: &str, payload: Bytes) -> EcsResult<playground_core_ecs::SystemResponse> {
    playground_core_ecs::system_command_access::send_to_system(target_system, command_type, payload).await
}

/// Helper to send JSON data to a system
pub async fn send_json_to_system<T: serde::Serialize>(
    target_system: &str, 
    command_type: &str, 
    data: &T
) -> EcsResult<playground_core_ecs::SystemResponse> {
    playground_core_ecs::system_command_access::send_json(target_system, command_type, data).await
}