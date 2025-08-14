mod handlers;
mod plugin;
mod state;

use playground_plugin::Plugin;
pub use plugin::PlaygroundEditor;
pub use state::EditorState;

#[no_mangle]
pub extern "C" fn create_plugin() -> Box<dyn Plugin> {
    Box::new(PlaygroundEditor::new())
}