//! VTable registration for systems/networking
//!
//! This module registers all networking handlers with the ECS VTable system

use tokio::sync::mpsc;
use playground_core_types::CoreResult;
use playground_core_ecs::{VTableCommand, get_world};
use crate::vtable_handlers;

/// Register all server VTable handlers
async fn register_server_handlers() -> CoreResult<()> {
    let world = get_world().await?;

    // Create channel for server operations
    let (tx, mut rx) = mpsc::channel::<VTableCommand>(100);

    // Register the channel with VTable
    world.vtable.register("server".to_string(), tx.clone()).await?;

    // Spawn handler task for server operations
    tokio::spawn(async move {
        while let Some(cmd) = rx.recv().await {
            let response = vtable_handlers::handle_server_operations(
                cmd.operation,
                cmd.payload
            ).await;
            let _ = cmd.response.send(response).await;
        }
    });

    // Create channel for channel operations if feature enabled
    #[cfg(feature = "channels")]
    {
        let (tx_channels, mut rx_channels) = mpsc::channel::<VTableCommand>(100);
        world.vtable.register("server.channels".to_string(), tx_channels).await?;

        tokio::spawn(async move {
            while let Some(cmd) = rx_channels.recv().await {
                let response = vtable_handlers::handle_channel_operations(
                    cmd.operation,
                    cmd.payload
                ).await;
                let _ = cmd.response.send(response).await;
            }
        });
    }

    // Create channel for server events
    let (tx_events, mut rx_events) = mpsc::channel::<VTableCommand>(100);
    world.vtable.register("server.events".to_string(), tx_events).await?;

    tokio::spawn(async move {
        while let Some(cmd) = rx_events.recv().await {
            let response = vtable_handlers::handle_server_events(
                cmd.operation,
                cmd.payload
            ).await;
            let _ = cmd.response.send(response).await;
        }
    });

    Ok(())
}

/// Register all client VTable handlers
async fn register_client_handlers() -> CoreResult<()> {
    let world = get_world().await?;

    // Create channel for client operations
    let (tx, mut rx) = mpsc::channel::<VTableCommand>(100);
    world.vtable.register("client".to_string(), tx).await?;

    tokio::spawn(async move {
        while let Some(cmd) = rx.recv().await {
            let response = vtable_handlers::handle_client_operations(
                cmd.operation,
                cmd.payload
            ).await;
            let _ = cmd.response.send(response).await;
        }
    });

    // Note: Rendering operations removed - they belong in systems/webgl
    // Note: Input operations removed - they belong in systems/input
    // Note: Audio operations removed - they belong in systems/audio

    Ok(())
}

/// Initialize networking system and register all handlers
/// This should be called during system startup
pub async fn initialize() -> CoreResult<()> {
    // Initialize the network state
    vtable_handlers::initialize().await?;

    // Register all handlers with the ECS VTable
    register_server_handlers().await?;
    register_client_handlers().await?;

    Ok(())
}