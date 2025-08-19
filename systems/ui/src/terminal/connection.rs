//! Terminal connection handler using core/server channels

use crate::error::{UiError, UiResult};
use crate::messages::{
    TerminalInputMessage, TerminalOutputMessage, TerminalConnectMessage,
    TerminalStateMessage, UiPacketType, serialize_message,
};
use crate::system::UiSystem;
use playground_core_server::Packet;
use std::sync::Arc;
use tokio::sync::{RwLock, mpsc};
use uuid::Uuid;

/// Terminal connection state
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TerminalState {
    Disconnected,
    Connecting,
    Connected,
    Ready,
    Error,
}

/// Terminal connection handler that uses core/server channels
pub struct TerminalConnection {
    terminal_id: Uuid,
    state: Arc<RwLock<TerminalState>>,
    ui_system: Arc<RwLock<UiSystem>>,
    input_tx: mpsc::Sender<String>,
    output_rx: Arc<RwLock<mpsc::Receiver<String>>>,
}

impl TerminalConnection {
    /// Create a new terminal connection
    pub fn new(ui_system: Arc<RwLock<UiSystem>>) -> Self {
        let (input_tx, _input_rx) = mpsc::channel(100);
        let (_output_tx, output_rx) = mpsc::channel(100);
        
        Self {
            terminal_id: Uuid::new_v4(),
            state: Arc::new(RwLock::new(TerminalState::Disconnected)),
            ui_system,
            input_tx,
            output_rx: Arc::new(RwLock::new(output_rx)),
        }
    }
    
    /// Get the terminal ID
    pub fn id(&self) -> Uuid {
        self.terminal_id
    }
    
    /// Connect to the terminal through core/server channels
    pub async fn connect(&self, shell_path: Option<String>, working_dir: Option<String>) -> UiResult<()> {
        // Update state
        *self.state.write().await = TerminalState::Connecting;
        
        // Send connect message through UI system
        let msg = TerminalConnectMessage {
            terminal_id: self.terminal_id,
            shell_path,
            working_dir,
        };
        
        let payload = serialize_message(&msg)?;
        let ui = self.ui_system.read().await;
        ui.send_packet(UiPacketType::TerminalConnect, payload).await?;
        
        // Update state to connected
        *self.state.write().await = TerminalState::Connected;
        
        Ok(())
    }
    
    /// Disconnect from the terminal
    pub async fn disconnect(&self) -> UiResult<()> {
        // Update state
        *self.state.write().await = TerminalState::Disconnected;
        
        // Send disconnect message through UI system  
        let msg = TerminalStateMessage {
            terminal_id: self.terminal_id,
            connected: false,
            ready: false,
        };
        
        let payload = serialize_message(&msg)?;
        let ui = self.ui_system.read().await;
        ui.send_packet(UiPacketType::TerminalDisconnect, payload).await?;
        
        Ok(())
    }
    
    /// Send input to the terminal
    pub async fn send_input(&self, input: String) -> UiResult<()> {
        // Check if connected
        let state = *self.state.read().await;
        if state != TerminalState::Connected && state != TerminalState::Ready {
            return Err(UiError::TerminalError("Terminal not connected".to_string()));
        }
        
        // Send through channel
        self.input_tx.send(input.clone()).await
            .map_err(|e| UiError::TerminalError(format!("Failed to queue input: {}", e)))?;
        
        // Send input message through UI system
        let msg = TerminalInputMessage {
            terminal_id: self.terminal_id,
            input,
        };
        
        let payload = serialize_message(&msg)?;
        let ui = self.ui_system.read().await;
        ui.send_packet(UiPacketType::TerminalInput, payload).await?;
        
        Ok(())
    }
    
    /// Process output from the terminal
    pub async fn receive_output(&self) -> Option<String> {
        let mut rx = self.output_rx.write().await;
        rx.recv().await
    }
    
    /// Handle incoming terminal output message
    pub async fn handle_output(&self, output: String, is_error: bool) -> UiResult<()> {
        // Send output message through UI system
        let msg = TerminalOutputMessage {
            terminal_id: self.terminal_id,
            output,
            is_error,
        };
        
        let payload = serialize_message(&msg)?;
        let ui = self.ui_system.read().await;
        ui.send_packet(UiPacketType::TerminalOutput, payload).await?;
        
        Ok(())
    }
    
    /// Get the current terminal state
    pub async fn get_state(&self) -> TerminalState {
        *self.state.read().await
    }
    
    /// Update the terminal state
    pub async fn set_state(&self, state: TerminalState) {
        *self.state.write().await = state;
    }
}

/// Terminal manager for handling multiple terminal connections
pub struct TerminalManager {
    connections: Arc<RwLock<Vec<Arc<TerminalConnection>>>>,
    ui_system: Arc<RwLock<UiSystem>>,
}

impl TerminalManager {
    /// Create a new terminal manager
    pub fn new(ui_system: Arc<RwLock<UiSystem>>) -> Self {
        Self {
            connections: Arc::new(RwLock::new(Vec::new())),
            ui_system,
        }
    }
    
    /// Create a new terminal connection
    pub async fn create_terminal(&self) -> Arc<TerminalConnection> {
        let connection = Arc::new(TerminalConnection::new(Arc::clone(&self.ui_system)));
        let mut connections = self.connections.write().await;
        connections.push(Arc::clone(&connection));
        connection
    }
    
    /// Get a terminal connection by ID
    pub async fn get_terminal(&self, id: Uuid) -> Option<Arc<TerminalConnection>> {
        let connections = self.connections.read().await;
        connections.iter()
            .find(|c| c.id() == id)
            .cloned()
    }
    
    /// Remove a terminal connection
    pub async fn remove_terminal(&self, id: Uuid) {
        let mut connections = self.connections.write().await;
        connections.retain(|c| c.id() != id);
    }
    
    /// Handle incoming packet from WebSocket
    pub async fn handle_packet(&self, packet: Packet) -> UiResult<()> {
        let packet_type = UiPacketType::try_from(packet.packet_type)?;
        
        match packet_type {
            UiPacketType::TerminalOutput => {
                let msg: TerminalOutputMessage = crate::messages::deserialize_message(&packet.payload)?;
                if let Some(terminal) = self.get_terminal(msg.terminal_id).await {
                    terminal.handle_output(msg.output, msg.is_error).await?;
                }
            }
            UiPacketType::TerminalState => {
                let msg: TerminalStateMessage = crate::messages::deserialize_message(&packet.payload)?;
                if let Some(terminal) = self.get_terminal(msg.terminal_id).await {
                    let state = if msg.connected && msg.ready {
                        TerminalState::Ready
                    } else if msg.connected {
                        TerminalState::Connected
                    } else {
                        TerminalState::Disconnected
                    };
                    terminal.set_state(state).await;
                }
            }
            _ => {}
        }
        
        Ok(())
    }
}