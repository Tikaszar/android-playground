use async_trait::async_trait;
use playground_plugin::Plugin;
use playground_types::{
    PluginMetadata, PluginId, Version, Event,
    context::Context,
    render_context::RenderContext,
    error::PluginError,
    Priority,
};
use playground_networking::NetworkingSystem;
use tracing::{info, debug, error};
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::panel_manager::PanelManager;
use crate::mcp_handler::McpHandler;
use crate::browser_bridge::BrowserBridge;
use crate::ui_state::UiState;
use crate::orchestrator::Orchestrator;

/// The UI Framework Plugin coordinates all UI updates between server plugins and the browser.
/// It listens for MCP tool calls and routes them to the appropriate UI panels.
pub struct UiFrameworkPlugin {
    metadata: PluginMetadata,
    panel_manager: Arc<RwLock<PanelManager>>,
    mcp_handler: Arc<McpHandler>,
    browser_bridge: Arc<BrowserBridge>,
    ui_state: Arc<RwLock<UiState>>,
    orchestrator: Arc<RwLock<Orchestrator>>,
    channel_id: Option<u16>,
    networking: Option<Arc<RwLock<NetworkingSystem>>>,
}

impl UiFrameworkPlugin {
    pub fn new() -> Self {
        // Create persistence directory for conversations
        let persistence_path = std::path::PathBuf::from("/data/data/com.termux/files/home/.android-playground/conversations");
        
        // Create UI state with persistence
        let ui_state = Arc::new(RwLock::new(UiState::with_persistence(persistence_path.clone())));
        let browser_bridge = Arc::new(BrowserBridge::new());
        let panel_manager = Arc::new(RwLock::new(PanelManager::new()));
        
        // Create orchestrator first
        let ui_state_clone = ui_state.clone();
        let orchestrator = Arc::new(RwLock::new(Orchestrator::new(
            tokio::task::block_in_place(|| {
                tokio::runtime::Handle::current().block_on(async {
                    ui_state_clone.read().await.channel_manager.clone()
                })
            }),
            tokio::task::block_in_place(|| {
                tokio::runtime::Handle::current().block_on(async {
                    ui_state_clone.read().await.message_system.clone()
                })
            }),
        )));
        
        // Create McpHandler with orchestrator
        let mut mcp_handler_inner = McpHandler::new(
            ui_state.clone(),
            browser_bridge.clone(),
            panel_manager.clone(),
        );
        mcp_handler_inner.set_orchestrator(orchestrator.clone());
        let mcp_handler = Arc::new(mcp_handler_inner);

        Self {
            metadata: PluginMetadata {
                id: PluginId("ui-framework".to_string()),
                name: "UI Framework".to_string(),
                version: Version {
                    major: 0,
                    minor: 1,
                    patch: 0,
                },
            },
            panel_manager,
            mcp_handler,
            browser_bridge,
            ui_state,
            orchestrator,
            channel_id: None,
            networking: None,
        }
    }
    
    async fn handle_mcp_tool_call(&mut self, tool_name: &str, arguments: serde_json::Value) {
        use crate::mcp_handler::ToolResult;
        
        // Process the tool call through MCP handler
        let result = self.mcp_handler.handle_tool_call(tool_name, arguments).await;
        
        match result {
            Ok(ToolResult { success, message }) => {
                if success {
                    info!("Tool {} executed successfully: {}", tool_name, message);
                } else {
                    error!("Tool {} failed: {}", tool_name, message);
                }
                
                // Send response back via channel 1201 if needed
                if let Some(networking) = &self.networking {
                    let response = serde_json::json!({
                        "type": "tool_result",
                        "tool": tool_name,
                        "success": success,
                        "message": message
                    });
                    
                    let data = serde_json::to_vec(&response).unwrap_or_default();
                    let net = networking.read().await;
                    let _ = net.send_packet(
                        1201, // Response channel
                        2, // Tool result packet type
                        data,
                        Priority::High
                    ).await;
                }
            }
            Err(e) => {
                error!("Failed to handle tool {}: {}", tool_name, e);
            }
        }
    }
}

#[async_trait]
impl Plugin for UiFrameworkPlugin {
    fn metadata(&self) -> &PluginMetadata {
        &self.metadata
    }

    async fn on_load(&mut self, ctx: &mut Context) -> Result<(), PluginError> {
        info!("UI Framework Plugin loading...");
        
        // Load previous conversations from disk
        {
            let channel_manager = {
                let ui_state = self.ui_state.read().await;
                ui_state.channel_manager.clone()
            };
            
            if let Err(e) = channel_manager.write().await.load_from_disk().await {
                info!("No previous conversations loaded: {}", e);
            } else {
                info!("Loaded previous conversations from disk");
            };
        }
        
        // Get NetworkingSystem from Context (provided by the App)
        // The App should have added this to the context resources
        let networking = ctx.resources.get("networking")
            .and_then(|r| r.downcast_ref::<Arc<RwLock<NetworkingSystem>>>())
            .ok_or_else(|| PluginError::InitFailed("NetworkingSystem not found in context".to_string()))?
            .clone();
        
        // Register for channels 1200-1209 as a plugin
        let channel_id = {
            let mut net = networking.write().await;
            net.register_plugin("ui-framework").await
                .map_err(|e| PluginError::InitFailed(e.to_string()))?
        };
        
        self.channel_id = Some(channel_id);
        self.networking = Some(networking);
        
        info!("UI Framework Plugin registered on channel {}", channel_id);
        
        // If no channels exist, initialize default setup
        {
            let mut ui_state = self.ui_state.write().await;
            if ui_state.channel_manager.read().await.list_channels().is_empty() {
                info!("No channels found, initializing default setup...");
                ui_state.initialize_default_setup().await
                    .map_err(|e| PluginError::InitFailed(e.to_string()))?;
            }
        }
        
        // Initialize orchestrator
        {
            let mut orchestrator = self.orchestrator.write().await;
            orchestrator.initialize().await
                .map_err(|e| PluginError::InitFailed(e.to_string()))?;
            
            // Start the assignment loop in the background
            let orchestrator_clone = self.orchestrator.clone();
            tokio::spawn(async move {
                let orchestrator = orchestrator_clone.read().await;
                orchestrator.run_assignment_loop().await;
            });
        }
        
        info!("UI Framework Plugin loaded successfully");
        Ok(())
    }

    async fn on_unload(&mut self, _ctx: &mut Context) {
        info!("UI Framework Plugin unloading...");
        
        // Cleanup resources
        if let Some(channel_id) = self.channel_id {
            debug!("Unregistering from channel {}", channel_id);
            // TODO: Unregister from channel manager
        }
    }

    async fn update(&mut self, _ctx: &mut Context, _delta_time: f32) {
        // Process any pending UI updates
        // This is called every frame
        
        // Check for MCP messages on our channel
        if let Some(channel_id) = self.channel_id {
            if let Some(networking) = &self.networking {
                // Receive packets from our channel (release lock quickly)
                let packets = {
                    let net = networking.read().await;
                    net.receive_packets(channel_id).await.unwrap_or_default()
                };
                
                // Process packets without holding the lock
                for packet in packets {
                    // Parse MCP tool call from packet
                    if let Ok(json) = serde_json::from_slice::<serde_json::Value>(&packet.data) {
                        debug!("Received MCP message on channel {}: {:?}", channel_id, json);
                        
                        // Handle MCP tool call
                        if json.get("type").and_then(|v| v.as_str()) == Some("tool_call") {
                            let tool_name = json.get("tool").and_then(|v| v.as_str()).unwrap_or("");
                            let arguments = json.get("arguments").cloned().unwrap_or(serde_json::json!({}));
                            
                            info!("Processing MCP tool call: {}", tool_name);
                            
                            // Process the tool call
                            self.handle_mcp_tool_call(tool_name, arguments).await;
                        }
                    }
                }
            }
        }
    }

    async fn render(&mut self, _ctx: &mut RenderContext) {
        // UI Framework doesn't render directly - it sends commands to browser
    }

    async fn on_event(&mut self, event: &Event) -> bool {
        // Handle events from other plugins or MCP
        debug!("Received event: {} with data: {:?}", event.id, event.data);
        
        // Check if this is an MCP event
        if event.id.starts_with("mcp:") {
            // TODO: Parse and handle MCP events
            true
        } else {
            false
        }
    }
}

#[no_mangle]
pub extern "C" fn create_plugin() -> Box<dyn Plugin> {
    Box::new(UiFrameworkPlugin::new())
}