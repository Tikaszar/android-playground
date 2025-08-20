use async_trait::async_trait;
use playground_systems_logic::{System, World, LogicResult};
use playground_core_types::{
    Priority,
};
use playground_core_ecs::Component;
use std::sync::Arc;
use std::collections::HashMap;
use tokio::sync::RwLock;
use tracing::{info, debug, error};
use uuid;

use crate::panel_manager::PanelManager;
use crate::mcp_handler::McpHandler;
use crate::browser_bridge::BrowserBridge;
use crate::ui_state::UiState;
use crate::orchestrator::Orchestrator;

/// The UI Framework Plugin coordinates all UI updates between server plugins and the browser.
/// It implements systems/logic::System so it can be registered in the World as a System.
pub struct UiFrameworkPlugin {
    panel_manager: Arc<RwLock<PanelManager>>,
    mcp_handler: Arc<McpHandler>,
    browser_bridge: Arc<BrowserBridge>,
    ui_state: Arc<RwLock<UiState>>,
    orchestrator: Arc<RwLock<Orchestrator>>,
    channel_id: Option<u16>,
    systems_manager: Arc<playground_systems_logic::SystemsManager>,
}

impl UiFrameworkPlugin {
    pub fn new(systems_manager: Arc<playground_systems_logic::SystemsManager>) -> Self {
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
        
        // Create MCP handler with references to all components
        let mut mcp_handler = McpHandler::new(
            ui_state.clone(),
            browser_bridge.clone(),
            panel_manager.clone(),
        );
        mcp_handler.set_orchestrator(orchestrator.clone());
        let mcp_handler = Arc::new(mcp_handler);
        
        Self {
            panel_manager,
            mcp_handler,
            browser_bridge,
            ui_state,
            orchestrator,
            channel_id: None,
            systems_manager,
        }
    }
}

#[async_trait]
impl System for UiFrameworkPlugin {
    fn name(&self) -> &'static str {
        "UiFrameworkPlugin"
    }
    
    async fn initialize(&mut self, _world: &World) -> LogicResult<()> {
        info!("UI Framework Plugin initializing...");
        
        // Register our channel (1200) with the networking system
        let networking = self.systems_manager.networking();
        let net = networking.read().await;
        
        // Register channels 1200-1209 for UI Framework
        for i in 0..10 {
            let channel = 1200 + i;
            net.register_plugin(&format!("ui-framework-{}", i)).await
                .map_err(|e| playground_systems_logic::LogicError::InitializationFailed(
                    format!("Failed to register channel {}: {}", channel, e)
                ))?;
        }
        
        self.channel_id = Some(1200);
        
        // Register MCP tools for UI manipulation
        self.register_mcp_tools().await?;
        
        // Start listening for messages on our channels
        self.start_message_listener().await;
        
        // Initialize panels
        let mut pm = self.panel_manager.write().await;
        pm.initialize_default_panels().await;
        
        // Create the Discord-style UI layout
        self.create_discord_ui().await?;
        
        info!("UI Framework Plugin initialized on channels 1200-1209");
        Ok(())
    }
    
    async fn run(&mut self, _world: &World, delta_time: f32) -> LogicResult<()> {
        // Process any pending UI updates
        let mut orchestrator = self.orchestrator.write().await;
        orchestrator.process_pending_updates(delta_time).await;
        
        // Check for incoming messages from browser
        if let Some(channel_id) = self.channel_id {
            let networking = self.systems_manager.networking();
            let net = networking.read().await;
            
            // Check all our channels (1200-1209)
            for i in 0..10 {
                let channel = channel_id + i;
                if let Ok(packets) = net.receive_packets(channel).await {
                    for packet in packets {
                        self.handle_browser_message(packet.data).await;
                    }
                }
            }
        }
        
        Ok(())
    }
    
    async fn cleanup(&mut self, _world: &World) -> LogicResult<()> {
        info!("UI Framework Plugin shutting down...");
        
        // Save UI state
        let ui_state = self.ui_state.read().await;
        ui_state.save_state().await
            .map_err(|e| playground_systems_logic::LogicError::SystemError(
                format!("Failed to save UI state: {}", e)
            ))?;
        
        Ok(())
    }
}

impl UiFrameworkPlugin {
    async fn register_mcp_tools(&self) -> LogicResult<()> {
        // Register MCP tools with the networking system
        let tools = vec![
            ("ui_create_panel", "Create a new UI panel", serde_json::json!({
                "type": "object",
                "properties": {
                    "panel_type": { "type": "string" },
                    "title": { "type": "string" },
                    "position": { "type": "object" }
                }
            })),
            ("ui_update_panel", "Update an existing UI panel", serde_json::json!({
                "type": "object",
                "properties": {
                    "panel_id": { "type": "string" },
                    "content": { "type": "object" }
                }
            })),
            ("ui_send_message", "Send a message to the chat", serde_json::json!({
                "type": "object",
                "properties": {
                    "channel_id": { "type": "string" },
                    "content": { "type": "string" },
                    "author": { "type": "string" }
                }
            })),
        ];
        
        for (name, description, schema) in tools {
            self.systems_manager.register_mcp_tool(
                name.to_string(),
                description.to_string(),
                schema,
                1200, // Our base channel
            ).await?;
        }
        
        Ok(())
    }
    
    async fn start_message_listener(&self) {
        // Start a task to listen for messages from the browser
        let mcp_handler = self.mcp_handler.clone();
        let orchestrator = self.orchestrator.clone();
        
        tokio::spawn(async move {
            debug!("UI Framework message listener started");
            // Message handling will be done in the run() method
        });
    }
    
    async fn handle_browser_message(&self, data: Vec<u8>) {
        // Parse and handle messages from the browser
        match serde_json::from_slice::<serde_json::Value>(&data) {
            Ok(msg) => {
                debug!("Received browser message: {:?}", msg);
                
                // Route to appropriate handler
                if let Some(msg_type) = msg.get("type").and_then(|v| v.as_str()) {
                    match msg_type {
                        "mcp_tool_call" => {
                            if let (Some(tool_name), Some(params)) = (
                                msg.get("tool_name").and_then(|v| v.as_str()),
                                msg.get("params")
                            ) {
                                match self.mcp_handler.handle_tool_call(tool_name, params.clone()).await {
                                    Ok(result) => {
                                        debug!("Tool call succeeded: {:?}", result);
                                    }
                                    Err(e) => {
                                        error!("Tool call failed: {}", e);
                                    }
                                }
                            } else {
                                error!("Invalid mcp_tool_call message: missing tool_name or params");
                            }
                        }
                        "panel_update" => {
                            let mut pm = self.panel_manager.write().await;
                            pm.handle_panel_update(msg).await;
                        }
                        "chat_message" => {
                            let mut ui_state = self.ui_state.write().await;
                            if let Err(e) = ui_state.handle_chat_message(msg).await {
                                error!("Failed to handle chat message: {}", e);
                            }
                        }
                        _ => {
                            debug!("Unknown message type: {}", msg_type);
                        }
                    }
                }
            }
            Err(e) => {
                error!("Failed to parse browser message: {}", e);
            }
        }
    }
    
    async fn create_discord_ui(&self) -> LogicResult<()> {
        // Get the UI system from SystemsManager
        let ui_system = self.systems_manager.ui();
        let mut ui = ui_system.write().await;
        
        // Get screen dimensions (default to 1920x1080, will be updated on resize)
        let screen_width = 1920.0;
        let screen_height = 1080.0;
        
        // Create sidebar for channel list (Discord-style, 240px width)
        let sidebar = ui.create_element(
            "sidebar".to_string(),
            "panel".to_string(),
            ui.root_entity(),
        ).await.map_err(|e| playground_systems_logic::LogicError::SystemError(
            format!("Failed to create sidebar: {}", e)
        ))?;
        
        // Set sidebar bounds and style
        let sidebar_style = playground_systems_ui::components::UiStyleComponent {
            theme_id: playground_systems_ui::theme::ThemeId(0),
            background_color: nalgebra::Vector4::new(0.184, 0.192, 0.212, 1.0), // #2f3136
            border_color: nalgebra::Vector4::new(0.125, 0.129, 0.145, 1.0), // #202225
            text_color: nalgebra::Vector4::new(0.863, 0.867, 0.871, 1.0), // #dcddde
            border_width: 0.0,
            border_radius: 0.0,
            opacity: 1.0,
            custom_properties: std::collections::HashMap::new(),
        };
        
        // Use add_component_raw with boxed component
        let component_box = Box::new(sidebar_style) as Box<dyn playground_core_ecs::Component>;
        let component_id = <playground_systems_ui::components::UiStyleComponent as playground_core_ecs::Component>::component_id();
        ui.world().add_component_raw(sidebar, component_box, component_id).await
            .map_err(|e| playground_systems_logic::LogicError::SystemError(
                format!("Failed to add sidebar style: {}", e)
            ))?;
        
        // Create channel categories
        let categories = vec!["SYSTEMS", "IDE PLUGINS", "GAME PLUGINS"];
        let mut y_offset = 20.0;
        
        for category in categories {
            let category_entity = ui.create_element(
                format!("category-{}", category),
                "text".to_string(),
                Some(sidebar),
            ).await.map_err(|e| playground_systems_logic::LogicError::SystemError(
                format!("Failed to create category: {}", e)
            ))?;
            
            // Add text component for category
            let text_component = playground_systems_ui::components::UiTextComponent {
                text: category.to_string(),
                font_family: "sans-serif".to_string(),
                font_size: 12.0,
                font_weight: playground_systems_ui::components::FontWeight::Bold,
                text_align: playground_systems_ui::components::TextAlign::Left,
                line_height: 1.5,
                letter_spacing: 0.0,
            };
            
            let component_box = Box::new(text_component) as Box<dyn playground_core_ecs::Component>;
            let component_id = <playground_systems_ui::components::UiTextComponent as playground_core_ecs::Component>::component_id();
            ui.world().add_component_raw(category_entity, component_box, component_id).await
                .map_err(|e| playground_systems_logic::LogicError::SystemError(
                    format!("Failed to add text component: {}", e)
                ))?;
            
            y_offset += 30.0;
        }
        
        // Create main content area
        let main_content = ui.create_element(
            "main-content".to_string(),
            "panel".to_string(),
            ui.root_entity(),
        ).await.map_err(|e| playground_systems_logic::LogicError::SystemError(
            format!("Failed to create main content: {}", e)
        ))?;
        
        // Set main content style (Discord chat area)
        let main_style = playground_systems_ui::components::UiStyleComponent {
            theme_id: playground_systems_ui::theme::ThemeId(0),
            background_color: nalgebra::Vector4::new(0.212, 0.224, 0.247, 1.0), // #36393f
            border_color: nalgebra::Vector4::new(0.125, 0.129, 0.145, 1.0),
            text_color: nalgebra::Vector4::new(0.863, 0.867, 0.871, 1.0),
            border_width: 0.0,
            border_radius: 0.0,
            opacity: 1.0,
            custom_properties: std::collections::HashMap::new(),
        };
        
        let component_box = Box::new(main_style) as Box<dyn playground_core_ecs::Component>;
        let component_id = <playground_systems_ui::components::UiStyleComponent as playground_core_ecs::Component>::component_id();
        ui.world().add_component_raw(main_content, component_box, component_id).await
            .map_err(|e| playground_systems_logic::LogicError::SystemError(
                format!("Failed to add main content style: {}", e)
            ))?;
        
        // Mark root as dirty to trigger initial render
        if let Some(root) = ui.root_entity() {
            ui.mark_dirty(root).await
                .map_err(|e| playground_systems_logic::LogicError::SystemError(
                    format!("Failed to mark root dirty: {}", e)
                ))?;
        }
        
        // Also mark sidebar and main content as dirty
        ui.mark_dirty(sidebar).await
            .map_err(|e| playground_systems_logic::LogicError::SystemError(
                format!("Failed to mark sidebar dirty: {}", e)
            ))?;
        
        ui.mark_dirty(main_content).await
            .map_err(|e| playground_systems_logic::LogicError::SystemError(
                format!("Failed to mark main content dirty: {}", e)
            ))?;
        
        info!("Discord-style UI layout created");
        Ok(())
    }
}