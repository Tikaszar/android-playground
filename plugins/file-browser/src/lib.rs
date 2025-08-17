mod plugin;
mod file_tree;
mod file_system;

pub use plugin::*;

#[no_mangle]
pub extern "C" fn create_plugin() -> Box<dyn playground_plugin::Plugin> {
    Box::new(plugin::create())
}
