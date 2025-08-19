use async_trait::async_trait;
use playground_core_plugin::Plugin;
use playground_core_types::{
    PluginMetadata, PluginId, Version, Event,
    context::Context,
    render_context::RenderContext,
    error::PluginError,
};

pub struct ChatPlugin {
    metadata: PluginMetadata,
}

impl ChatPlugin {
    pub fn new() -> Self {
        Self {
            metadata: PluginMetadata {
                id: PluginId("chat".to_string()),
                name: "Chat".to_string(),
                version: Version {
                    major: 0,
                    minor: 1,
                    patch: 0,
                },
            },
        }
    }
}

#[async_trait]
impl Plugin for ChatPlugin {
    fn metadata(&self) -> &PluginMetadata {
        &self.metadata
    }

    async fn on_load(&mut self, _ctx: &mut Context) -> Result<(), PluginError> {
        Ok(())
    }

    async fn on_unload(&mut self, _ctx: &mut Context) {
        // Cleanup if needed
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

pub fn create() -> ChatPlugin {
    ChatPlugin::new()
}
