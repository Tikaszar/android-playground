#![allow(improper_ctypes_definitions)]

use playground_types::context::Context;
use playground_types::error::PluginError;
use playground_types::event::Event;
use playground_types::plugin_metadata::PluginMetadata;
use playground_types::render_context::RenderContext;

pub trait Plugin: Send + Sync + 'static {
    fn metadata(&self) -> &PluginMetadata;

    fn on_load(&mut self, ctx: &mut Context) -> Result<(), PluginError>;
    fn on_unload(&mut self, ctx: &mut Context);
    fn update(&mut self, ctx: &mut Context, delta_time: f32);
    fn render(&mut self, ctx: &mut RenderContext);

    fn on_event(&mut self, event: &Event) -> bool;
}

pub type CreatePluginFn = unsafe extern "C" fn() -> *mut dyn Plugin;