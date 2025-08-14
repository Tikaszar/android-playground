use playground_types::context::Context;
use playground_types::error::PluginError;
use playground_types::event::Event;
use playground_types::render_context::RenderContext;
use playground_types::server::plugin::PluginInfo;

pub trait Plugin: Send + Sync + 'static {
    fn info(&self) -> PluginInfo;

    fn on_load(&mut self, ctx: &mut Context) -> Result<(), PluginError>;
    fn on_unload(&mut self, ctx: &mut Context);
    fn update(&mut self, ctx: &mut Context, delta_time: f32);
    fn render(&mut self, ctx: &mut RenderContext);

    fn on_event(&mut self, event: &Event) -> bool;
}

pub trait Stateful {
    fn save_state(&self) -> serde_json::Value;
    fn load_state(&mut self, state: serde_json::Value);
}

pub type CreatePluginFn = unsafe extern "C" fn() -> Box<dyn Plugin>;
