use serde::{Deserialize, Serialize};

#[derive(Default, Serialize, Deserialize)]
pub struct EditorState {
    pub current_file: Option<String>,
    pub cursor_position: (usize, usize),
    pub selection: Option<(usize, usize)>,
    pub open_files: Vec<String>,
}