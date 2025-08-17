mod plugin;
mod chat_view;
mod mcp_client;

pub use plugin::*;

#[no_mangle]
pub extern "C" fn create_plugin() -> Box<dyn playground_plugin::Plugin> {
    Box::new(plugin::create())
}
