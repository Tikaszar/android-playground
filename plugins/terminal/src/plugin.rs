use playground_core_plugin::Plugin;
use playground_core_types::{
    PluginMetadata, PluginId, Version, Event,
    context::Context,
    render_context::RenderContext,
    error::PluginError,
};
use tracing::info;

pub struct TerminalPlugin {
    metadata: PluginMetadata,
    channel_id: Option<u16>,
}

impl TerminalPlugin {
    pub fn new() -> Self {
        Self {
            metadata: PluginMetadata {
                id: PluginId("terminal".to_string()),
                name: "Terminal".to_string(),
                version: Version {
                    major: 0,
                    minor: 1,
                    patch: 0,
                },
            },
            channel_id: None,
        }
    }
}

impl Plugin for TerminalPlugin {
    fn metadata(&self) -> &PluginMetadata {
        &self.metadata
    }

    fn on_load(&mut self, _ctx: &mut Context) -> Result<(), PluginError> {
        info!("terminal plugin loading");
        
        // Register with networking system for appropriate channels
        // self.channel_id = Some(CHANNEL_BASE);
        
        info!("terminal plugin loaded successfully");
        Ok(())
    }

    fn on_unload(&mut self, _ctx: &mut Context) {
        info!("terminal plugin unloading");
    }

    fn update(&mut self, _ctx: &mut Context, _delta_time: f32) {
        // Update logic here
    }

    fn render(&mut self, _ctx: &mut RenderContext) {
        // Render logic here
    }

    fn on_event(&mut self, _event: &Event) -> bool {
        // Handle events, return true if handled
        false
    }
}

pub fn create() -> TerminalPlugin {
    TerminalPlugin::new()
}
