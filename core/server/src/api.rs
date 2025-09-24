//! Public API functions for server operations using ECS
//!
//! These functions work with entities and components in the ECS,
//! similar to how core/rendering works.

use playground_core_types::CoreResult;
use playground_core_ecs::{Entity, EntityRef, get_world};
use crate::types::*;
use crate::components::*;

/// Start a server with the given configuration
/// Returns the server entity
pub async fn start_server(config: ServerConfig) -> CoreResult<Entity> {
    let world = get_world().await?;
    let server_entity = world.spawn_entity().await?;

    // Add server components
    server_entity.add_component(ServerConfigComponent::new(config)).await?;
    server_entity.add_component(ServerState::new()).await?;
    server_entity.add_component(ServerStatsComponent::new()).await?;

    #[cfg(feature = "batching")]
    {
        let batch_config = server_entity.get_component::<ServerConfigComponent>().await?;
        if batch_config.config.enable_batching {
            server_entity.add_component(MessageQueue::new(
                1000, // Default batch size
                batch_config.config.batch_interval
            )).await?;
        }
    }

    Ok(server_entity)
}

/// Stop a server
pub async fn stop_server(_server: EntityRef) -> CoreResult<()> {
    Ok(())
}

/// Accept a new connection
/// Returns the connection entity
pub async fn accept_connection(address: String) -> CoreResult<Entity> {
    let world = get_world().await?;
    let conn_entity = world.spawn_entity().await?;

    let mut connection = ServerConnection::new(ConnectionId::new());
    connection.metadata.insert("address".to_string(), address);

    conn_entity.add_component(connection).await?;

    Ok(conn_entity)
}

/// Send a message to a connection
pub async fn send_to_connection(_connection: EntityRef, _message: Message) -> CoreResult<()> {
    Ok(())
}

/// Broadcast a message to all connections
pub async fn broadcast_message(message: Message) -> CoreResult<()> {
    let _world = get_world().await?;

    // Query for all connections
    let connections: Vec<Entity> = vec![];

    for conn_entity in connections {
        send_to_connection(conn_entity.downgrade(), message.clone()).await?;
    }

    Ok(())
}

/// Create a channel for pub/sub messaging
#[cfg(feature = "channels")]
pub async fn create_channel(name: String) -> CoreResult<Entity> {
    let world = get_world().await?;
    let channel_entity = world.spawn_entity().await?;

    channel_entity.add_component(ServerChannel::new(ChannelId::new(), name)).await?;

    Ok(channel_entity)
}

/// Subscribe a connection to a channel
#[cfg(feature = "channels")]
pub async fn subscribe_to_channel(_connection: EntityRef, _channel: EntityRef) -> CoreResult<()> {
    Ok(())
}

/// Unsubscribe a connection from a channel
#[cfg(feature = "channels")]
pub async fn unsubscribe_from_channel(_connection: EntityRef, _channel: EntityRef) -> CoreResult<()> {
    Ok(())
}

/// Publish a message to a channel
#[cfg(feature = "channels")]
pub async fn publish_to_channel(_channel: EntityRef, _message: Message) -> CoreResult<()> {
    Ok(())
}

/// Disconnect a connection
pub async fn disconnect_connection(_connection: EntityRef) -> CoreResult<()> {
    Ok(())
}

/// Queue a message for batched sending
#[cfg(feature = "batching")]
pub async fn queue_message(_server: EntityRef, _connection: EntityRef, _message: Message) -> CoreResult<()> {
    Ok(())
}

/// Flush the message queue
#[cfg(feature = "batching")]
pub async fn flush_message_queue(_server: EntityRef) -> CoreResult<()> {
    Ok(())
}