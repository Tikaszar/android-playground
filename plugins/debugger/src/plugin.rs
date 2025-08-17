use playground_plugin::{Plugin, PluginContext};
use playground_types::{PluginMetadata, Event};
use uuid::Uuid;

pub struct DebuggerPlugin {
    id: Uuid,
    metadata: PluginMetadata,
}

impl DebuggerPlugin {
    pub fn new() -> Self {
        Self {
            id: Uuid::new_v4(),
            metadata: PluginMetadata {
                name: "debugger".to_string(),
                version: "0.1.0".to_string(),
                author: "Playground Team".to_string(),
                description: "Debugger plugin".to_string(),
            },
        }
    }
}

impl Plugin for DebuggerPlugin {
    fn id(&self) -> Uuid {
        self.id
    }

    fn metadata(&self) -> &PluginMetadata {
        &self.metadata
    }

    fn on_load(&mut self, _context: &mut PluginContext) -> Result<(), Box<dyn std::error::Error>> {
        Ok(())
    }

    fn on_unload(&mut self, _context: &mut PluginContext) -> Result<(), Box<dyn std::error::Error>> {
        Ok(())
    }

    fn update(&mut self, _context: &mut PluginContext, _delta_time: f32) -> Result<(), Box<dyn std::error::Error>> {
        Ok(())
    }

    fn render(&mut self, _context: &mut PluginContext) -> Result<(), Box<dyn std::error::Error>> {
        Ok(())
    }

    fn on_event(&mut self, _context: &mut PluginContext, _event: Event) -> Result<(), Box<dyn std::error::Error>> {
        Ok(())
    }
}

pub fn create() -> DebuggerPlugin {
    DebuggerPlugin::new()
}
