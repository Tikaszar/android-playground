use anyhow::Result;
use bytes::{Bytes, BytesMut, BufMut};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::{mpsc, RwLock};
use tracing::{info, debug, error, warn};
use uuid::Uuid;

use crate::browser_bridge::{BrowserBridge, BrowserUpdate};
use crate::channel_manager::ChannelManager;
use crate::message_system::MessageSystem;
use crate::mcp_handler::McpHandler;
use crate::ui_state::UiState;

/// Handles WebSocket communication for the UI Framework Plugin
pub struct WebSocketHandler {
    ui_state: Arc<RwLock<UiState>>,
    channel_manager: Arc<RwLock<ChannelManager>>,
    message_system: Arc<MessageSystem>,
    browser_bridge: Arc<BrowserBridge>,
    mcp_handler: Arc<McpHandler>,
    
    // Channel IDs
    ui_channel_base: u16,
    ui_channel_results: u16,
    
    // Message queues
    tx: mpsc::UnboundedSender<ChannelMessage>,
    rx: Arc<RwLock<mpsc::UnboundedReceiver<ChannelMessage>>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChannelMessage {
    pub channel_id: u16,
    pub packet_type: u16,
    pub payload: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum UiFrameworkMessage {
    // From browser to server
    GetState,
    SendMessage {
        channel: String,
        content: String,
        timestamp: u64,
    },
    BubbleStateChange {
        component_id: String,
        state: String,
    },
    SaveFile {
        filepath: String,
        content: String,
    },
    OpenFile {
        filepath: String,
    },
    ToggleDirectory {
        path: String,
    },
    
    // From server to browser
    StateUpdate {
        state: serde_json::Value,
    },
    Message {
        message: MessageData,
    },
    ChannelUpdate {
        channel: ChannelData,
    },
    InlineComponent {
        component: ComponentData,
    },
    AgentStatus {
        status: String,
    },
    Error {
        error: String,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageData {
    pub id: Uuid,
    pub channel_id: String,
    pub author: String,
    pub author_type: String, // "user", "assistant", "system"
    pub content: String,
    pub timestamp: u64,
    pub components: Vec<ComponentData>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentData {
    pub id: Uuid,
    #[serde(rename = "type")]
    pub component_type: String,
    pub title: Option<String>,
    pub initial_state: String,
    pub data: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChannelData {
    pub id: String,
    pub name: String,
    pub channel_type: String,
    pub unread_count: u32,
}

impl WebSocketHandler {
    pub fn new(
        ui_state: Arc<RwLock<UiState>>,
        channel_manager: Arc<RwLock<ChannelManager>>,
        message_system: Arc<MessageSystem>,
        browser_bridge: Arc<BrowserBridge>,
        mcp_handler: Arc<McpHandler>,
    ) -> Self {
        let (tx, rx) = mpsc::unbounded_channel();
        
        Self {
            ui_state,
            channel_manager,
            message_system,
            browser_bridge,
            mcp_handler,
            ui_channel_base: 1200,
            ui_channel_results: 1201,
            tx,
            rx: Arc::new(RwLock::new(rx)),
        }
    }
    
    /// Initialize WebSocket channels
    pub async fn initialize(&self) -> Result<()> {
        info!("Initializing UI Framework WebSocket channels");
        
        // Register for channels 1200-1209
        // In a real implementation, this would connect to the core/server channel manager
        
        // Set up initial state
        let mut ui_state = self.ui_state.write().await;
        ui_state.initialize_default_setup().await?;
        
        info!("UI Framework WebSocket handler initialized");
        Ok(())
    }
    
    /// Handle incoming packet from WebSocket
    pub async fn handle_packet(&self, channel_id: u16, packet_type: u16, payload: Bytes) -> Result<()> {
        debug!("Received packet on channel {}: type {}", channel_id, packet_type);
        
        // Parse the payload
        let message: UiFrameworkMessage = serde_json::from_slice(&payload)?;
        
        match message {
            UiFrameworkMessage::GetState => {
                self.handle_get_state().await?;
            }
            UiFrameworkMessage::SendMessage { channel, content, timestamp } => {
                self.handle_send_message(channel, content, timestamp).await?;
            }
            UiFrameworkMessage::BubbleStateChange { component_id, state } => {
                self.handle_bubble_state_change(component_id, state).await?;
            }
            UiFrameworkMessage::SaveFile { filepath, content } => {
                self.handle_save_file(filepath, content).await?;
            }
            UiFrameworkMessage::OpenFile { filepath } => {
                self.handle_open_file(filepath).await?;
            }
            UiFrameworkMessage::ToggleDirectory { path } => {
                self.handle_toggle_directory(path).await?;
            }
            _ => {
                warn!("Unexpected message type from browser");
            }
        }
        
        Ok(())
    }
    
    /// Handle MCP tool call from LLM
    pub async fn handle_mcp_tool_call(&self, tool_name: &str, params: serde_json::Value) -> Result<()> {
        debug!("Handling MCP tool call: {}", tool_name);
        
        // Forward to MCP handler
        self.mcp_handler.handle_tool_call(tool_name, params).await?;
        
        // Get any browser updates that resulted
        let updates = self.browser_bridge.flush_updates().await?;
        
        // Send updates to browser
        for update in updates {
            self.send_browser_update(update).await?;
        }
        
        Ok(())
    }
    
    // Message handlers
    
    async fn handle_get_state(&self) -> Result<()> {
        debug!("Handling get_state request");
        
        let ui_state = self.ui_state.read().await;
        let channel_manager = self.channel_manager.read().await;
        
        // Build state object
        let state = serde_json::json!({
            "channels": channel_manager.list_channels(),
            "agents": channel_manager.list_agents(),
            "tasks": ui_state.task_queue,
        });
        
        self.send_to_browser(UiFrameworkMessage::StateUpdate { state }).await?;
        
        Ok(())
    }
    
    async fn handle_send_message(&self, channel: String, content: String, timestamp: u64) -> Result<()> {
        debug!("Handling send_message: channel={}, content={}", channel, content);
        
        // Create message
        let message_id = Uuid::new_v4();
        let message = MessageData {
            id: message_id,
            channel_id: channel.clone(),
            author: "User".to_string(),
            author_type: "user".to_string(),
            content: content.clone(),
            timestamp,
            components: vec![],
        };
        
        // Store in channel manager
        let channel_id = Uuid::parse_str(&channel).unwrap_or_else(|_| Uuid::new_v4());
        let sender = crate::components::AgentId(Uuid::new_v4()); // User agent
        
        // Use the channel_manager through proper async locking
        let mut channel_manager = self.channel_manager.write().await;
        channel_manager.send_message(
            channel_id,
            sender,
            crate::components::MessageContent::Text(content),
        ).await?;
        
        // Send to browser
        self.send_to_browser(UiFrameworkMessage::Message { message }).await?;
        
        // TODO: Forward to appropriate agent/LLM
        
        Ok(())
    }
    
    async fn handle_bubble_state_change(&self, component_id: String, state: String) -> Result<()> {
        debug!("Handling bubble_state_change: component={}, state={}", component_id, state);
        
        // Update state in UI state manager
        let mut ui_state = self.ui_state.write().await;
        
        // Parse state
        let bubble_state = match state.as_str() {
            "collapsed" => crate::components::BubbleState::Collapsed,
            "compressed" => crate::components::BubbleState::Compressed,
            _ => crate::components::BubbleState::Expanded,
        };
        
        // TODO: Update component state
        
        Ok(())
    }
    
    async fn handle_save_file(&self, filepath: String, content: String) -> Result<()> {
        debug!("Handling save_file: {}", filepath);
        
        // TODO: Actually save the file
        // For now, just send a notification
        self.browser_bridge.show_notification(
            "File Saved".to_string(),
            format!("Saved {}", filepath)
        ).await?;
        
        Ok(())
    }
    
    async fn handle_open_file(&self, filepath: String) -> Result<()> {
        debug!("Handling open_file: {}", filepath);
        
        // TODO: Read file and create editor component
        // For now, create a mock editor
        let component = ComponentData {
            id: Uuid::new_v4(),
            component_type: "editor".to_string(),
            title: Some(filepath.clone()),
            initial_state: "expanded".to_string(),
            data: serde_json::json!({
                "filepath": filepath,
                "content": "// File content would go here",
                "language": "rust",
            }),
        };
        
        self.send_to_browser(UiFrameworkMessage::InlineComponent { component }).await?;
        
        Ok(())
    }
    
    async fn handle_toggle_directory(&self, path: String) -> Result<()> {
        debug!("Handling toggle_directory: {}", path);
        
        // TODO: Update file browser state
        
        Ok(())
    }
    
    // Utility functions
    
    async fn send_to_browser(&self, message: UiFrameworkMessage) -> Result<()> {
        let payload = serde_json::to_value(&message)?;
        
        let channel_message = ChannelMessage {
            channel_id: self.ui_channel_base,
            packet_type: 1, // Data packet
            payload,
        };
        
        self.tx.send(channel_message)?;
        Ok(())
    }
    
    async fn send_browser_update(&self, update: BrowserUpdate) -> Result<()> {
        // Convert BrowserUpdate to UiFrameworkMessage
        let message = match update {
            BrowserUpdate::UpdateStatus { message } => {
                UiFrameworkMessage::AgentStatus { status: message }
            }
            BrowserUpdate::ShowNotification { title, message } => {
                // Create a system message
                UiFrameworkMessage::Message {
                    message: MessageData {
                        id: Uuid::new_v4(),
                        channel_id: "system".to_string(),
                        author: title,
                        author_type: "system".to_string(),
                        content: message,
                        timestamp: chrono::Utc::now().timestamp_millis() as u64,
                        components: vec![],
                    }
                }
            }
            _ => {
                // For other updates, send as raw JSON
                UiFrameworkMessage::StateUpdate {
                    state: serde_json::to_value(&update)?
                }
            }
        };
        
        self.send_to_browser(message).await
    }
    
    /// Get pending messages to send
    pub async fn get_pending_messages(&self) -> Vec<ChannelMessage> {
        let mut messages = vec![];
        let mut rx = self.rx.write().await;
        
        while let Ok(msg) = rx.try_recv() {
            messages.push(msg);
        }
        
        messages
    }
    
    /// Create a packet for sending
    pub fn create_packet(&self, channel_id: u16, payload: &serde_json::Value) -> Result<Bytes> {
        let payload_text = serde_json::to_string(payload)?;
        let payload_bytes = payload_text.as_bytes();
        
        let mut buffer = BytesMut::with_capacity(9 + payload_bytes.len());
        
        // Header
        buffer.put_u16_le(channel_id);
        buffer.put_u16_le(1); // packet_type: data
        buffer.put_u8(2); // priority: medium
        buffer.put_u32_le(payload_bytes.len() as u32);
        
        // Payload
        buffer.put_slice(payload_bytes);
        
        Ok(buffer.freeze())
    }
}