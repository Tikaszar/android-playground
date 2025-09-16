//! Server operation delegation methods
//! 
//! All methods delegate to systems/networking via VTable

use bytes::Bytes;
use playground_core_types::{CoreResult, CoreError};
use crate::{Server, ServerConfig, ServerStats, ConnectionId, ConnectionInfo, Message, ChannelId};

impl Server {
    /// Start the server (delegated to systems/networking via VTable)
    pub async fn start(&self, config: ServerConfig) -> CoreResult<()> {
        let payload = bincode::serialize(&config)
            .map_err(|e| CoreError::SerializationError(e.to_string()))?;
        
        let response = self.vtable.send_command(
            "server",
            "start".to_string(),
            Bytes::from(payload)
        ).await?;
        
        if !response.success {
            return Err(CoreError::Generic(response.error.unwrap_or_else(|| "Failed to start server".to_string())));
        }
        
        Ok(())
    }
    
    /// Stop the server (delegated to systems/networking via VTable)
    pub async fn stop(&self) -> CoreResult<()> {
        let response = self.vtable.send_command(
            "server",
            "stop".to_string(),
            Bytes::new()
        ).await?;
        
        if !response.success {
            return Err(CoreError::Generic(response.error.unwrap_or_else(|| "Failed to stop server".to_string())));
        }
        
        Ok(())
    }
    
    /// Check if the server is running
    pub async fn is_running(&self) -> bool {
        *self.is_running.read().await
    }
    
    /// Get server statistics
    pub async fn stats(&self) -> ServerStats {
        self.stats.read().await.clone()
    }
    
    /// Get server configuration
    pub async fn config(&self) -> ServerConfig {
        self.config.read().await.clone()
    }
    
    /// Send a message to a specific connection (delegated via VTable)
    pub async fn send_to(&self, connection: ConnectionId, message: Message) -> CoreResult<()> {
        #[derive(serde::Serialize)]
        struct SendToPayload {
            connection: ConnectionId,
            message: Message,
        }
        
        let payload = bincode::serialize(&SendToPayload { connection, message })
            .map_err(|e| CoreError::SerializationError(e.to_string()))?;
        
        let response = self.vtable.send_command(
            "server",
            "send_to".to_string(),
            Bytes::from(payload)
        ).await?;
        
        if !response.success {
            return Err(CoreError::Generic(response.error.unwrap_or_else(|| "Failed to send message".to_string())));
        }
        
        Ok(())
    }
    
    /// Broadcast a message to all connections (delegated via VTable)
    pub async fn broadcast(&self, message: Message) -> CoreResult<()> {
        let payload = bincode::serialize(&message)
            .map_err(|e| CoreError::SerializationError(e.to_string()))?;
        
        let response = self.vtable.send_command(
            "server",
            "broadcast".to_string(),
            Bytes::from(payload)
        ).await?;
        
        if !response.success {
            return Err(CoreError::Generic(response.error.unwrap_or_else(|| "Failed to broadcast message".to_string())));
        }
        
        Ok(())
    }
    
    /// Publish a message to a channel (delegated via VTable)
    #[cfg(feature = "channels")]
    pub async fn publish(&self, channel: ChannelId, message: Message) -> CoreResult<()> {
        #[derive(serde::Serialize)]
        struct PublishPayload {
            channel: ChannelId,
            message: Message,
        }
        
        let payload = bincode::serialize(&PublishPayload { channel, message })
            .map_err(|e| CoreError::SerializationError(e.to_string()))?;
        
        let response = self.vtable.send_command(
            "server",
            "publish".to_string(),
            Bytes::from(payload)
        ).await?;
        
        if !response.success {
            return Err(CoreError::Generic(response.error.unwrap_or_else(|| "Failed to publish message".to_string())));
        }
        
        Ok(())
    }
    
    /// Subscribe a connection to a channel (delegated via VTable)
    #[cfg(feature = "channels")]
    pub async fn subscribe(&self, connection: ConnectionId, channel: ChannelId) -> CoreResult<()> {
        #[derive(serde::Serialize)]
        struct SubscribePayload {
            connection: ConnectionId,
            channel: ChannelId,
        }
        
        let payload = bincode::serialize(&SubscribePayload { connection, channel })
            .map_err(|e| CoreError::SerializationError(e.to_string()))?;
        
        let response = self.vtable.send_command(
            "server.channels",
            "subscribe".to_string(),
            Bytes::from(payload)
        ).await?;
        
        if !response.success {
            return Err(CoreError::Generic(response.error.unwrap_or_else(|| "Failed to subscribe to channel".to_string())));
        }
        
        Ok(())
    }
    
    /// Unsubscribe a connection from a channel (delegated via VTable)
    #[cfg(feature = "channels")]
    pub async fn unsubscribe(&self, connection: ConnectionId, channel: ChannelId) -> CoreResult<()> {
        #[derive(serde::Serialize)]
        struct UnsubscribePayload {
            connection: ConnectionId,
            channel: ChannelId,
        }
        
        let payload = bincode::serialize(&UnsubscribePayload { connection, channel })
            .map_err(|e| CoreError::SerializationError(e.to_string()))?;
        
        let response = self.vtable.send_command(
            "server.channels",
            "unsubscribe".to_string(),
            Bytes::from(payload)
        ).await?;
        
        if !response.success {
            return Err(CoreError::Generic(response.error.unwrap_or_else(|| "Failed to unsubscribe from channel".to_string())));
        }
        
        Ok(())
    }
    
    /// Get list of active connections
    pub async fn connections(&self) -> Vec<ConnectionInfo> {
        self.connections.read().await.values().cloned().collect()
    }
    
    /// Get connection info
    pub async fn connection(&self, id: ConnectionId) -> Option<ConnectionInfo> {
        self.connections.read().await.get(&id).cloned()
    }
    
    /// Handle incoming connection (called by implementation)
    pub async fn on_connection(&self, connection: ConnectionInfo) -> CoreResult<()> {
        let payload = bincode::serialize(&connection)
            .map_err(|e| CoreError::SerializationError(e.to_string()))?;
        
        let response = self.vtable.send_command(
            "server.events",
            "on_connection".to_string(),
            Bytes::from(payload)
        ).await?;
        
        if !response.success {
            return Err(CoreError::Generic(response.error.unwrap_or_else(|| "Failed to handle connection".to_string())));
        }
        
        Ok(())
    }
    
    /// Handle connection closed (called by implementation)
    pub async fn on_disconnection(&self, id: ConnectionId) -> CoreResult<()> {
        let payload = bincode::serialize(&id)
            .map_err(|e| CoreError::SerializationError(e.to_string()))?;
        
        let response = self.vtable.send_command(
            "server.events",
            "on_disconnection".to_string(),
            Bytes::from(payload)
        ).await?;
        
        if !response.success {
            return Err(CoreError::Generic(response.error.unwrap_or_else(|| "Failed to handle disconnection".to_string())));
        }
        
        Ok(())
    }
}