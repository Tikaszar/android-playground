use playground_plugin::Plugin;
use playground_types::{
    PluginMetadata, PluginId, Version, Event,
    context::Context,
    render_context::RenderContext,
    error::PluginError,
};
use tracing::{info, debug, error};
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::panel_manager::PanelManager;
use crate::mcp_handler::McpHandler;
use crate::browser_bridge::BrowserBridge;
use crate::ui_state::UiState;

/// The UI Framework Plugin coordinates all UI updates between server plugins and the browser.
/// It listens for MCP tool calls and routes them to the appropriate UI panels.
pub struct UiFrameworkPlugin {
    metadata: PluginMetadata,
    panel_manager: Arc<RwLock<PanelManager>>,
    mcp_handler: Arc<McpHandler>,
    browser_bridge: Arc<BrowserBridge>,
    ui_state: Arc<RwLock<UiState>>,
    channel_id: Option<u16>,
}

impl UiFrameworkPlugin {
    pub fn new() -> Self {
        let ui_state = Arc::new(RwLock::new(UiState::new()));
        let browser_bridge = Arc::new(BrowserBridge::new());
        let panel_manager = Arc::new(RwLock::new(PanelManager::new()));
        let mcp_handler = Arc::new(McpHandler::new(
            ui_state.clone(),
            browser_bridge.clone(),
            panel_manager.clone(),
        ));

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
            channel_id: None,
        }
    }
}

impl Plugin for UiFrameworkPlugin {
    fn metadata(&self) -> &PluginMetadata {
        &self.metadata
    }

    fn on_load(&mut self, context: &mut Context) -> Result<(), PluginError> {
        info!("UI Framework Plugin loading...");
        
        // Register for channel 1200-1209
        let channel_id = 1200;
        self.channel_id = Some(channel_id);
        
        // TODO: Register with channel manager when Context provides access
        // context.channel_manager.register(channel_id, "ui-framework")?;
        
        info!("UI Framework Plugin registered on channel {}", channel_id);
        
        // Initialize browser bridge connection
        // This will establish WebSocket connection on channel 10 for UI updates
        
        info!("UI Framework Plugin loaded successfully");
        Ok(())
    }

    fn on_unload(&mut self, _context: &mut Context) -> Result<(), PluginError> {
        info!("UI Framework Plugin unloading...");
        
        // Cleanup resources
        if let Some(channel_id) = self.channel_id {
            debug!("Unregistering from channel {}", channel_id);
            // TODO: Unregister from channel manager
        }
        
        Ok(())
    }

    fn update(&mut self, context: &mut Context, _delta_time: f32) -> Result<(), PluginError> {
        // Process any pending UI updates
        // This is called every frame
        
        // Check for MCP messages on our channel
        if let Some(channel_id) = self.channel_id {
            // TODO: Read messages from channel when Context provides access
            // let messages = context.channel_manager.read_channel(channel_id)?;
            // for message in messages {
            //     self.mcp_handler.handle_message(message)?;
            // }
        }
        
        Ok(())
    }

    fn render(&mut self, _context: &mut RenderContext) -> Result<(), PluginError> {
        // UI Framework doesn't render directly - it sends commands to browser
        Ok(())
    }

    fn on_event(&mut self, _context: &mut Context, event: Event) -> Result<bool, PluginError> {
        match event {
            Event::Custom(data) => {
                // Handle custom events from other plugins or MCP
                debug!("Received custom event: {:?}", data);
                // TODO: Parse and handle the event
                Ok(true)
            }
            _ => Ok(false),
        }
    }
}

#[no_mangle]
pub extern "C" fn create_plugin() -> Box<dyn Plugin> {
    Box::new(UiFrameworkPlugin::new())
}