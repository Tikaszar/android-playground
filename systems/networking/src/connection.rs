//! Connection management utilities

use crate::{NetworkError, NetworkResult};
// ServerHandle will be defined locally
use std::sync::Arc;
use tokio::sync::RwLock;

/// Connection state for WebSocket
#[derive(Debug, Clone)]
pub enum ConnectionState {
    Disconnected,
    Connecting,
    Connected,
    Reconnecting,
    Failed(String),
}

/// Connection manager for WebSocket connections
pub struct ConnectionManager {
    state: Arc<RwLock<ConnectionState>>,
    server_url: String,
    reconnect_attempts: u32,
    max_reconnect_attempts: u32,
}

impl ConnectionManager {
    pub fn new(server_url: String) -> Self {
        Self {
            state: Arc::new(RwLock::new(ConnectionState::Disconnected)),
            server_url,
            reconnect_attempts: 0,
            max_reconnect_attempts: 10,
        }
    }
    
    /// Connect to the WebSocket server
    pub async fn connect(&mut self) -> NetworkResult<()> {
        let mut state = self.state.write().await;
        *state = ConnectionState::Connecting;
        
        // TODO: Implement actual WebSocket connection to core/server
        // For now, return a placeholder
        
        *state = ConnectionState::Connected;
        
        // Create a dummy server handle for now
        // In real implementation, this would be the actual connection
        Ok(())
    }
    
    /// Reconnect with exponential backoff
    pub async fn reconnect(&mut self) -> NetworkResult<()> {
        if self.reconnect_attempts >= self.max_reconnect_attempts {
            let mut state = self.state.write().await;
            *state = ConnectionState::Failed(format!(
                "Max reconnection attempts ({}) exceeded",
                self.max_reconnect_attempts
            ));
            return Err(NetworkError::ConnectionFailed(
                "Max reconnection attempts exceeded".to_string()
            ));
        }
        
        let mut state = self.state.write().await;
        *state = ConnectionState::Reconnecting;
        drop(state);
        
        // Exponential backoff
        let delay_ms = 100 * (2_u32.pow(self.reconnect_attempts.min(10)));
        tokio::time::sleep(tokio::time::Duration::from_millis(delay_ms as u64)).await;
        
        self.reconnect_attempts += 1;
        
        match self.connect().await {
            Ok(()) => {
                self.reconnect_attempts = 0;
                Ok(())
            }
            Err(e) => {
                // Try again
                Err(e)
            }
        }
    }
    
    /// Get current connection state
    pub async fn get_state(&self) -> ConnectionState {
        self.state.read().await.clone()
    }
    
    /// Check if connected
    pub async fn is_connected(&self) -> bool {
        matches!(*self.state.read().await, ConnectionState::Connected)
    }
    
    /// Disconnect
    pub async fn disconnect(&mut self) -> NetworkResult<()> {
        let mut state = self.state.write().await;
        *state = ConnectionState::Disconnected;
        self.reconnect_attempts = 0;
        
        // TODO: Close actual WebSocket connection
        
        Ok(())
    }
}