use async_trait::async_trait;
use playground_systems_logic::{System, World, LogicResult, SystemsManager, UiInterface, Component};
use playground_core_types::{
    Priority, Shared, shared, Handle, handle,
};
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
    panel_manager: Shared<PanelManager>,
    mcp_handler: Handle<McpHandler>,
    browser_bridge: Handle<BrowserBridge>,
    ui_state: Shared<UiState>,
    orchestrator: Shared<Orchestrator>,
    channel_id: Option<u16>,
    systems_manager: Handle<playground_systems_logic::SystemsManager>,
}

impl UiFrameworkPlugin {
    pub fn new(systems_manager: Handle<playground_systems_logic::SystemsManager>) -> Self {
        // Create persistence directory for conversations
        let persistence_path = std::path::PathBuf::from("/data/data/com.termux/files/home/.android-playground/conversations");
        
        // Create UI state with persistence
        let ui_state = shared(UiState::with_persistence(persistence_path.clone()));
        let browser_bridge = handle(BrowserBridge::new());
        let panel_manager = shared(PanelManager::new());
        
        // Create orchestrator first
        let ui_state_clone = ui_state.clone();
        let orchestrator = shared(Orchestrator::new(
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
        ));
        
        // Create MCP handler with references to all components
        let mut mcp_handler = McpHandler::new(
            ui_state.clone(),
            browser_bridge.clone(),
            panel_manager.clone(),
        );
        mcp_handler.set_orchestrator(orchestrator.clone());
        let mcp_handler = handle(mcp_handler);
        
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
        // Log to dashboard
        self.systems_manager.log("info", "[UI-FW] UI Framework Plugin initialize() called".to_string()).await;
        
        // Request dynamic channel for UI Framework
        self.systems_manager.log("info", "[UI-FW] Requesting dynamic channel allocation...".to_string()).await;
        
        let channel = self.systems_manager.register_plugin("ui-framework").await?;
        self.channel_id = Some(channel);
        
        self.systems_manager.log("info", format!("[UI-FW] Dynamic channel allocated: {}", channel)).await;
        
        // Register with networking system
        let networking = self.systems_manager.networking();
        let net = networking.read().await;
        
        // Register the allocated channel with networking
        if let Err(e) = net.register_system_channel("ui-framework", channel).await {
            self.systems_manager.log("error", format!("[UI-FW] Failed to register channel {}: {}", channel, e)).await;
            return Err(playground_systems_logic::LogicError::InitializationFailed(
                format!("Failed to register channel {}: {}", channel, e)
            ));
        }
        
        self.systems_manager.log("info", "[UI-FW] Channel registered with networking successfully".to_string()).await;
        
        // Register MCP tools for UI manipulation
        self.systems_manager.log("info", "[UI-FW] Registering MCP tools...".to_string()).await;
        self.register_mcp_tools().await?;
        self.systems_manager.log("info", "[UI-FW] MCP tools registered".to_string()).await;
        
        // Start listening for messages on our channels
        self.systems_manager.log("info", "[UI-FW] Starting message listener...".to_string()).await;
        self.start_message_listener().await;
        self.systems_manager.log("info", "[UI-FW] Message listener started".to_string()).await;
        
        // Initialize panels
        self.systems_manager.log("info", "[UI-FW] Initializing panels...".to_string()).await;
        let mut pm = self.panel_manager.write().await;
        pm.initialize_default_panels().await;
        drop(pm);
        self.systems_manager.log("info", "[UI-FW] Panels initialized".to_string()).await;
        
        // Create the Discord-style UI layout
        self.systems_manager.log("info", "[UI-FW] Creating Discord UI...".to_string()).await;
        self.create_discord_ui().await?;
        self.systems_manager.log("info", "[UI-FW] Discord UI created successfully".to_string()).await;
        
        self.systems_manager.log("info", "[UI-FW] UI Framework Plugin initialize() completed".to_string()).await;
        
        Ok(())
    }
    
    async fn run(&mut self, _world: &World, delta_time: f32) -> LogicResult<()> {
        // Process any pending UI updates
        let mut orchestrator = self.orchestrator.write().await;
        orchestrator.process_pending_updates(delta_time).await;
        
        // Check for incoming messages from browser on our dynamic channel
        if let Some(channel_id) = self.channel_id {
            let networking = self.systems_manager.networking();
            let net = networking.read().await;
            
            // Check our dynamically allocated channel
            if let Ok(packets) = net.receive_packets(channel_id).await {
                for packet in packets {
                    self.handle_packet(packet.packet_type, packet.data).await;
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
                self.channel_id.unwrap_or(1), // Use dynamic base channel
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
    
    async fn handle_packet(&self, packet_type: u16, data: Vec<u8>) {
        use crate::packet_types::*;
        
        match packet_type {
            PACKET_TYPE_MCP_TOOL_CALL => self.handle_mcp_tool_call(data).await,
            PACKET_TYPE_PANEL_UPDATE => self.handle_panel_update(data).await,
            PACKET_TYPE_CHAT_MESSAGE => self.handle_chat_message(data).await,
            _ => {
                debug!("Unknown packet type {} received on UI Framework channel", packet_type);
            }
        }
    }
    
    async fn handle_mcp_tool_call(&self, data: Vec<u8>) {
        match serde_json::from_slice::<serde_json::Value>(&data) {
            Ok(msg) => {
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
            Err(e) => {
                error!("Failed to parse MCP tool call: {}", e);
            }
        }
    }
    
    async fn handle_panel_update(&self, data: Vec<u8>) {
        match serde_json::from_slice::<serde_json::Value>(&data) {
            Ok(msg) => {
                let mut pm = self.panel_manager.write().await;
                pm.handle_panel_update(msg).await;
            }
            Err(e) => {
                error!("Failed to parse panel update: {}", e);
            }
        }
    }
    
    async fn handle_chat_message(&self, data: Vec<u8>) {
        match serde_json::from_slice::<serde_json::Value>(&data) {
            Ok(msg) => {
                let mut ui_state = self.ui_state.write().await;
                if let Err(e) = ui_state.handle_chat_message(msg).await {
                    error!("Failed to handle chat message: {}", e);
                }
            }
            Err(e) => {
                error!("Failed to parse chat message: {}", e);
            }
        }
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
        self.systems_manager.log("info", "[UI-FW] create_discord_ui() called".to_string()).await;
        
        // Get the UI interface from SystemsManager
        self.systems_manager.log("info", "[UI-FW] Getting UI interface...".to_string()).await;
        let mut ui_interface = self.systems_manager.ui_interface();
        self.systems_manager.log("info", "[UI-FW] Got UI interface".to_string()).await;
        
        // Create mobile Discord layout optimized for phones
        self.systems_manager.log("info", "[UI-FW] Calling create_mobile_discord_layout()...".to_string()).await;
        let layout = match ui_interface.create_mobile_discord_layout().await {
            Ok(layout) => {
                self.systems_manager.log("info", "[UI-FW] ✓ Successfully created Discord layout".to_string()).await;
                layout
            },
            Err(e) => {
                self.systems_manager.log("error", format!("[UI-FW] ✗ Failed to create Discord layout: {}", e)).await;
                // For now, just return Ok to not block initialization
                // The UI won't be created but the plugin will still run
                return Ok(());
            }
        };
        
        // Add a header bar to the main content
        let header = ui_interface.create_panel(
            "header-bar",
            Some(layout.main_content),
        ).await?;
        
        ui_interface.set_bounds(
            header,
            playground_systems_ui::ElementBounds::new(0.0, 0.0, 360.0, 50.0)
        ).await?;
        
        // Add hamburger menu button (to show channel drawer)
        let menu_button = ui_interface.create_button(
            "☰",
            Some(header),
        ).await?;
        
        ui_interface.set_bounds(
            menu_button,
            playground_systems_ui::ElementBounds::new(10.0, 10.0, 30.0, 30.0)
        ).await?;
        
        // Add channel name in header
        let channel_name = ui_interface.create_text(
            "# general",
            Some(header),
        ).await?;
        
        ui_interface.style_element(channel_name, playground_systems_ui::ElementStyle {
            text_color: [0.863, 0.867, 0.871, 1.0],
            font_size: 18.0,
            font_weight: playground_systems_ui::FontWeight::Bold,
            ..Default::default()
        }).await?;
        
        ui_interface.set_bounds(
            channel_name,
            playground_systems_ui::ElementBounds::new(50.0, 15.0, 200.0, 30.0)
        ).await?;
        
        // Add channels to the drawer (off-screen initially)
        let channels = vec![
            ("SYSTEMS", vec!["# ui", "# networking", "# logic"]),
            ("IDE", vec!["# editor", "# terminal", "# files"]),
            ("CHAT", vec!["# general", "# help", "# announcements"]),
        ];
        
        let mut y_offset = 20.0;
        for (category, channel_list) in channels {
            // Category header
            let category_elem = ui_interface.create_text(
                category,
                Some(layout.sidebar),
            ).await?;
            
            ui_interface.style_element(category_elem, playground_systems_ui::ElementStyle {
                text_color: [0.54, 0.56, 0.60, 1.0],
                font_size: 12.0,
                font_weight: playground_systems_ui::FontWeight::Bold,
                ..Default::default()
            }).await?;
            
            ui_interface.set_bounds(
                category_elem,
                playground_systems_ui::ElementBounds::new(15.0, y_offset, 250.0, 20.0)
            ).await?;
            
            y_offset += 25.0;
            
            // Channels in category
            for channel_name in channel_list {
                let channel_elem = ui_interface.create_button(
                    channel_name,
                    Some(layout.sidebar),
                ).await?;
                
                ui_interface.style_element(channel_elem, playground_systems_ui::ElementStyle {
                    background_color: [0.0, 0.0, 0.0, 0.0], // Transparent
                    text_color: [0.7, 0.7, 0.7, 1.0],
                    font_size: 16.0, // Larger for mobile touch
                    ..Default::default()
                }).await?;
                
                ui_interface.set_bounds(
                    channel_elem,
                    playground_systems_ui::ElementBounds::new(20.0, y_offset, 240.0, 40.0)
                ).await?;
                
                y_offset += 42.0;
            }
            
            y_offset += 10.0; // Space between categories
        }
        
        // Add some initial messages to the message area
        ui_interface.add_message(
            layout.message_area,
            "System",
            "Welcome to the Android Playground IDE!",
            "now",
        ).await?;
        
        ui_interface.add_message(
            layout.message_area,
            "UI Framework",
            "Mobile Discord UI initialized successfully",
            "now",
        ).await?;
        
        // Add input field to the input area
        let input_field = ui_interface.create_panel(
            "message-input",
            Some(layout.input_area),
        ).await?;
        
        ui_interface.style_element(input_field, playground_systems_ui::ElementStyle {
            background_color: [0.251, 0.263, 0.286, 1.0],
            border_radius: 20.0, // Rounded like Discord mobile
            ..Default::default()
        }).await?;
        
        ui_interface.set_bounds(
            input_field,
            playground_systems_ui::ElementBounds::new(10.0, 10.0, 340.0, 40.0)
        ).await?;
        
        // Add placeholder text
        let placeholder = ui_interface.create_text(
            "Message #general",
            Some(input_field),
        ).await?;
        
        ui_interface.style_element(placeholder, playground_systems_ui::ElementStyle {
            text_color: [0.5, 0.5, 0.5, 1.0],
            font_size: 16.0,
            ..Default::default()
        }).await?;
        
        ui_interface.set_bounds(
            placeholder,
            playground_systems_ui::ElementBounds::new(20.0, 12.0, 300.0, 20.0)
        ).await?;
        
        // Force initial layout calculation
        ui_interface.force_layout().await?;
        
        // Store layout reference in UI state
        let mut ui_state = self.ui_state.write().await;
        // In production, you'd store these IDs in components
        drop(ui_state);
        
        info!("Mobile Discord UI layout created successfully");
        Ok(())
    }
}