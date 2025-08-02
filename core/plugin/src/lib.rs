use playground_types::{Context, Event, RenderContext, PluginError};
use std::error::Error;

pub trait Plugin: Send + Sync + 'static {
    fn id(&self) -> &str;
    fn name(&self) -> &str;
    fn version(&self) -> &str;
    
    fn dependencies(&self) -> Vec<&str> {
        vec![]
    }
    
    fn on_load(&mut self, ctx: &mut Context) -> Result<(), Box<dyn Error>>;
    fn on_unload(&mut self, ctx: &mut Context);
    fn update(&mut self, ctx: &mut Context, delta_time: f32);
    fn render(&mut self, ctx: &mut RenderContext);
    
    fn on_event(&mut self, event: &Event) -> bool {
        false
    }
}

pub trait Stateful {
    fn save_state(&self) -> serde_json::Value;
    fn load_state(&mut self, state: serde_json::Value);
}

pub type CreatePluginFn = unsafe extern "C" fn() -> Box<dyn Plugin>;