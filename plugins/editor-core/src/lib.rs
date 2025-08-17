mod plugin;
mod state;
mod buffer;
mod vim;
mod editor_view;

pub use plugin::EditorCorePlugin;
pub use state::{EditorState, OpenFile, CursorPosition};
pub use buffer::TextBuffer;
pub use vim::{VimState, VimMode, VimCommand, Direction, Motion};
pub use editor_view::EditorView;

/// Plugin entry point - required for dynamic loading
#[no_mangle]
pub extern "C" fn create_plugin() -> Box<dyn playground_plugin::Plugin> {
    Box::new(EditorCorePlugin::new())
}