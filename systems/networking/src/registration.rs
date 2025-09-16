//! VTable registration for systems/networking
//! 
//! This module registers all networking handlers with the Server and Client VTables

use tokio::sync::mpsc;
use playground_core_types::{Handle, CoreResult};
use playground_core_server::Server;
use playground_core_client::Client;
use playground_core_ecs::VTableCommand;
use crate::vtable_handlers;

/// Register all server VTable handlers
pub async fn register_server_handlers(server: Handle<Server>) -> CoreResult<()> {
    // Initialize handler storage
    vtable_handlers::initialize_handlers().await;
    
    // Create channel for server operations
    let (tx, mut rx) = mpsc::channel::<VTableCommand>(100);
    
    // Register the channel with VTable
    server.vtable.register("server".to_string(), tx.clone()).await?;
    
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
        server.vtable.register("server.channels".to_string(), tx_channels).await?;
        
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
    server.vtable.register("server.events".to_string(), tx_events).await?;
    
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
pub async fn register_client_handlers(client: Handle<Client>) -> CoreResult<()> {
    // Create channel for client operations
    let (tx, mut rx) = mpsc::channel::<VTableCommand>(100);
    client.vtable.register("client".to_string(), tx).await?;
    
    tokio::spawn(async move {
        while let Some(cmd) = rx.recv().await {
            let response = vtable_handlers::handle_client_operations(
                cmd.operation,
                cmd.payload
            ).await;
            let _ = cmd.response.send(response).await;
        }
    });
    
    // Register render operations if feature enabled
    #[cfg(feature = "rendering")]
    {
        let (tx_render, mut rx_render) = mpsc::channel::<VTableCommand>(100);
        client.vtable.register("client.render".to_string(), tx_render).await?;
        
        tokio::spawn(async move {
            while let Some(cmd) = rx_render.recv().await {
                let response = vtable_handlers::handle_render_operations(
                    cmd.operation,
                    cmd.payload
                ).await;
                let _ = cmd.response.send(response).await;
            }
        });
    }
    
    // Register input operations if feature enabled
    #[cfg(feature = "input")]
    {
        let (tx_input, mut rx_input) = mpsc::channel::<VTableCommand>(100);
        client.vtable.register("client.input".to_string(), tx_input).await?;
        
        tokio::spawn(async move {
            while let Some(cmd) = rx_input.recv().await {
                let response = vtable_handlers::handle_input_operations(
                    cmd.operation,
                    cmd.payload
                ).await;
                let _ = cmd.response.send(response).await;
            }
        });
    }
    
    // Register audio operations if feature enabled
    #[cfg(feature = "audio")]
    {
        let (tx_audio, mut rx_audio) = mpsc::channel::<VTableCommand>(100);
        client.vtable.register("client.audio".to_string(), tx_audio).await?;
        
        tokio::spawn(async move {
            while let Some(cmd) = rx_audio.recv().await {
                let response = vtable_handlers::handle_audio_operations(
                    cmd.operation,
                    cmd.payload
                ).await;
                let _ = cmd.response.send(response).await;
            }
        });
    }
    
    Ok(())
}

/// Initialize networking system and register all handlers
/// This should be called during system startup
pub async fn initialize() -> CoreResult<()> {
    // Get the global Server instance and register handlers - NOT async
    if let Ok(server) = playground_core_server::get_server_instance() {
        register_server_handlers(server.clone()).await?;
    }
    
    // Get the global Client instance and register handlers - NOT async
    if let Ok(client) = playground_core_client::get_client_instance() {
        register_client_handlers(client.clone()).await?;
    }
    
    Ok(())
}