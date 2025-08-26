mod plugin;
mod state;
mod buffer;
mod vim;
// mod editor_view; // TODO: Update to use new UI APIs

pub use plugin::EditorCorePlugin;
pub use state::{EditorState, OpenFile, CursorPosition};
pub use buffer::TextBuffer;
pub use vim::{VimState, VimMode, VimCommand, Direction, Motion};
// pub use editor_view::EditorView;