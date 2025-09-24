//! Public API functions for client operations using ECS
//!
//! These functions work with entities and components in the ECS,
//! similar to how core/rendering works.

use playground_core_types::CoreResult;
use playground_core_ecs::{Entity, EntityRef, get_world};
use crate::types::*;
use crate::components::*;
use crate::input::*;

/// Initialize a client with the given configuration
/// Returns the client entity
pub async fn initialize_client(config: ClientConfig) -> CoreResult<Entity> {
    let world = get_world().await?;
    let client_entity = world.spawn_entity().await?;

    // Add core client components
    let client_id = config.id;
    client_entity.add_component(ClientConfigComponent::new(config)).await?;
    client_entity.add_component(ClientStateComponent::new(client_id)).await?;
    client_entity.add_component(ClientStatsComponent::new()).await?;

    // Add optional components based on features
    #[cfg(feature = "input")]
    client_entity.add_component(InputStateComponent::new()).await?;

    #[cfg(feature = "audio")]
    client_entity.add_component(AudioStateComponent::new()).await?;

    Ok(client_entity)
}

/// Connect client to server
pub async fn connect_to_server(_client: EntityRef, _address: String) -> CoreResult<()> {
    Ok(())
}

/// Disconnect client from server
pub async fn disconnect_client(_client: EntityRef) -> CoreResult<()> {
    Ok(())
}

/// Update client stats
pub async fn update_client_fps(_client: EntityRef, _fps: Float, _frame_time_ms: Float) -> CoreResult<()> {
    Ok(())
}

/// Create a render target
#[cfg(feature = "rendering")]
pub async fn create_render_target(width: UInt, height: UInt) -> CoreResult<Entity> {
    let world = get_world().await?;
    let target_entity = world.spawn_entity().await?;

    let mut id_counter = 0;
    id_counter += 1;  // Simple ID generation

    target_entity.add_component(RenderTargetComponent::new(id_counter, width, height)).await?;

    Ok(target_entity)
}

/// Resize render target
#[cfg(feature = "rendering")]
pub async fn resize_render_target(_target: EntityRef, _width: UInt, _height: UInt) -> CoreResult<()> {
    Ok(())
}

/// Handle input event
#[cfg(feature = "input")]
pub async fn handle_input_event(_client: EntityRef, _event: InputEvent) -> CoreResult<()> {
    Ok(())
}

/// Check if key is pressed
#[cfg(feature = "input")]
pub async fn is_key_pressed(client: EntityRef, key: KeyCode) -> CoreResult<bool> {
    if let Some(input) = client.get_component::<InputStateComponent>().await {
        Ok(input.is_key_pressed(key))
    } else {
        Ok(false)
    }
}

/// Set master volume
#[cfg(feature = "audio")]
pub async fn set_master_volume(_client: EntityRef, _volume: Float) -> CoreResult<()> {
    Ok(())
}

/// Set music volume
#[cfg(feature = "audio")]
pub async fn set_music_volume(_client: EntityRef, _volume: Float) -> CoreResult<()> {
    Ok(())
}

/// Toggle mute
#[cfg(feature = "audio")]
pub async fn toggle_mute(_client: EntityRef) -> CoreResult<()> {
    Ok(())
}

/// Send message to server
pub async fn send_to_server(_client: EntityRef, _message: Vec<u8>) -> CoreResult<()> {
    Ok(())
}

/// Handle received message
pub async fn handle_received_message(_client: EntityRef, _message: Vec<u8>) -> CoreResult<()> {
    Ok(())
}