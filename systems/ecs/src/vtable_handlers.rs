//! VTable command handlers for ECS operations
//! 
//! This registers all ECS operations with the World's VTable so that
//! core/ecs can delegate operations to systems/ecs.

use bytes::Bytes;
use tokio::sync::mpsc;
use playground_core_ecs::{
    get_world, VTableCommand, VTableResponse,
    EntityId, ComponentId, Component, Generation,
    CoreError, CoreResult
};
use playground_core_types::Handle;
use crate::world_impl::WorldImpl;
use crate::storage_impl::StorageImpl;

/// Register all ECS handlers with the World's VTable
pub async fn register_handlers() -> CoreResult<()> {
    let world = get_world().await?;
    
    // Register entity operations
    register_entity_handlers(&world).await?;
    
    // Register component operations
    register_component_handlers(&world).await?;
    
    // Register query operations
    register_query_handlers(&world).await?;
    
    Ok(())
}

/// Register entity operation handlers
async fn register_entity_handlers(world: &Handle<playground_core_ecs::World>) -> CoreResult<()> {
    let (tx, mut rx) = mpsc::channel::<VTableCommand>(100);
    
    // Register the channel with VTable
    world.vtable.register("ecs.entity".to_string(), tx).await?;
    
    // Spawn handler task
    let world_handle = world.clone();
    tokio::spawn(async move {
        while let Some(cmd) = rx.recv().await {
            let result = handle_entity_command(&world_handle, cmd.operation, cmd.payload).await;
            
            let response = match result {
                Ok(payload) => VTableResponse {
                    success: true,
                    payload: Some(payload),
                    error: None,
                },
                Err(e) => VTableResponse {
                    success: false,
                    payload: None,
                    error: Some(e.to_string()),
                },
            };
            
            let _ = cmd.response.send(response).await;
        }
    });
    
    Ok(())
}

/// Handle entity commands
async fn handle_entity_command(world: &Handle<playground_core_ecs::World>, operation: String, payload: Bytes) -> CoreResult<Bytes> {
    match operation.as_str() {
        "spawn" => {
            let entity = WorldImpl::spawn_entity(world).await?;
            let bytes = bincode::serialize(&entity)
                .map_err(|e| CoreError::SerializationError(e.to_string()))?;
            Ok(Bytes::from(bytes))
        },
        "despawn" => {
            let entity: EntityId = bincode::deserialize(&payload)
                .map_err(|e| CoreError::DeserializationError(e.to_string()))?;
            WorldImpl::despawn_entity(world, entity).await?;
            Ok(Bytes::new())
        },
        _ => Err(CoreError::Generic(format!("Unknown entity operation: {}", operation)))
    }
}

/// Register component operation handlers
async fn register_component_handlers(world: &Handle<playground_core_ecs::World>) -> CoreResult<()> {
    let (tx, mut rx) = mpsc::channel::<VTableCommand>(100);
    
    // Register the channel with VTable
    world.vtable.register("ecs.component".to_string(), tx).await?;
    
    // Spawn handler task
    let world_handle = world.clone();
    tokio::spawn(async move {
        while let Some(cmd) = rx.recv().await {
            let result = handle_component_command(&world_handle, cmd.operation, cmd.payload).await;
            
            let response = match result {
                Ok(payload) => VTableResponse {
                    success: true,
                    payload: Some(payload),
                    error: None,
                },
                Err(e) => VTableResponse {
                    success: false,
                    payload: None,
                    error: Some(e.to_string()),
                },
            };
            
            let _ = cmd.response.send(response).await;
        }
    });
    
    Ok(())
}

/// Handle component commands
async fn handle_component_command(world: &Handle<playground_core_ecs::World>, operation: String, payload: Bytes) -> CoreResult<Bytes> {
    match operation.as_str() {
        "add" => {
            // Parse the custom serialization format we used in world.rs
            let bytes = payload.as_ref();
            let mut cursor = 0;
            
            // Read entity length and data
            let entity_len = u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]) as usize;
            cursor += 4;
            let entity: EntityId = bincode::deserialize(&bytes[cursor..cursor+entity_len])
                .map_err(|e| CoreError::DeserializationError(e.to_string()))?;
            cursor += entity_len;
            
            // Read component_id length and data
            let component_id_len = u32::from_le_bytes([bytes[cursor], bytes[cursor+1], bytes[cursor+2], bytes[cursor+3]]) as usize;
            cursor += 4;
            let component_id: ComponentId = bincode::deserialize(&bytes[cursor..cursor+component_id_len])
                .map_err(|e| CoreError::DeserializationError(e.to_string()))?;
            cursor += component_id_len;
            
            // Read component data length and data
            let data_len = u32::from_le_bytes([bytes[cursor], bytes[cursor+1], bytes[cursor+2], bytes[cursor+3]]) as usize;
            cursor += 4;
            let data = Bytes::copy_from_slice(&bytes[cursor..cursor+data_len]);
            cursor += data_len;
            
            // Read component name
            let name = String::from_utf8_lossy(&bytes[cursor..]).to_string();
            
            // Reconstruct the component
            let component = Component {
                data,
                component_id,
                component_name: name,
                size_hint: data_len,
            };
            
            WorldImpl::add_component(world, entity, component).await?;
            Ok(Bytes::new())
        },
        "remove" => {
            let (entity, component_id): (EntityId, ComponentId) = bincode::deserialize(&payload)
                .map_err(|e| CoreError::DeserializationError(e.to_string()))?;
            WorldImpl::remove_component(world, entity, component_id).await?;
            Ok(Bytes::new())
        },
        "get" => {
            let (entity, component_id): (EntityId, ComponentId) = bincode::deserialize(&payload)
                .map_err(|e| CoreError::DeserializationError(e.to_string()))?;
            let component = WorldImpl::get_component(world, entity, component_id).await?;
            // Return just the component data - the caller already knows the component_id
            Ok(component.data)
        },
        _ => Err(CoreError::Generic(format!("Unknown component operation: {}", operation)))
    }
}

/// Register query operation handlers
async fn register_query_handlers(world: &Handle<playground_core_ecs::World>) -> CoreResult<()> {
    let (tx, mut rx) = mpsc::channel::<VTableCommand>(100);
    
    // Register the channel with VTable
    world.vtable.register("ecs.query".to_string(), tx).await?;
    
    // Spawn handler task
    let world_handle = world.clone();
    tokio::spawn(async move {
        while let Some(cmd) = rx.recv().await {
            let result = handle_query_command(&world_handle, cmd.operation, cmd.payload).await;
            
            let response = match result {
                Ok(payload) => VTableResponse {
                    success: true,
                    payload: Some(payload),
                    error: None,
                },
                Err(e) => VTableResponse {
                    success: false,
                    payload: None,
                    error: Some(e.to_string()),
                },
            };
            
            let _ = cmd.response.send(response).await;
        }
    });
    
    Ok(())
}

/// Handle query commands
async fn handle_query_command(world: &Handle<playground_core_ecs::World>, operation: String, payload: Bytes) -> CoreResult<Bytes> {
    match operation.as_str() {
        "execute" => {
            let (required, excluded): (Vec<ComponentId>, Vec<ComponentId>) = bincode::deserialize(&payload)
                .map_err(|e| CoreError::DeserializationError(e.to_string()))?;
            let results = WorldImpl::query(world, required, excluded).await?;
            let bytes = bincode::serialize(&results)
                .map_err(|e| CoreError::SerializationError(e.to_string()))?;
            Ok(Bytes::from(bytes))
        },
        _ => Err(CoreError::Generic(format!("Unknown query operation: {}", operation)))
    }
}