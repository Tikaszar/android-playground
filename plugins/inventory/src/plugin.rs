use async_trait::async_trait;
use playground_core_plugin::Plugin;
use playground_core_types::{
    PluginMetadata, PluginId, Version, Event,
    context::Context,
    render_context::RenderContext,
    error::PluginError,
};
use tracing::info;

pub struct InventoryPlugin {
    metadata: PluginMetadata,
    channel_id: Option<u16>,
}

impl InventoryPlugin {
    pub fn new() -> Self {
        Self {
            metadata: PluginMetadata {
                id: PluginId("inventory".to_string()),
                name: "Inventory".to_string(),
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

#[async_trait]
impl Plugin for InventoryPlugin {
    fn metadata(&self) -> &PluginMetadata {
        &self.metadata
    }

    async fn on_load(&mut self, _ctx: &mut Context) -> Result<(), PluginError> {
        info!("inventory plugin loading");
        
        // Register with networking system for appropriate channels
        // self.channel_id = Some(CHANNEL_BASE);
        
        info!("inventory plugin loaded successfully");
        Ok(())
    }

    async fn on_unload(&mut self, _ctx: &mut Context) {
        info!("inventory plugin unloading");
    }

    async fn update(&mut self, _ctx: &mut Context, _delta_time: f32) {
        // Update logic here
    }

    async fn render(&mut self, _ctx: &mut RenderContext) {
        // Render logic here
    }

    async fn on_event(&mut self, _event: &Event) -> bool {
        // Handle events, return true if handled
        false
    }
}

pub fn create() -> InventoryPlugin {
    InventoryPlugin::new()
}
