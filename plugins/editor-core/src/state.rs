use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EditorState {
    pub open_files: Vec<OpenFile>,
    pub active_file: Option<usize>,
    pub vim_mode: bool,
    pub cursors: Vec<CursorPosition>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenFile {
    pub path: String,
    pub content: String,
    pub language: String,
    pub modified: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CursorPosition {
    pub line: usize,
    pub column: usize,
}

impl Default for EditorState {
    fn default() -> Self {
        Self {
            open_files: Vec::new(),
            active_file: None,
            vim_mode: false,
            cursors: vec![CursorPosition { line: 0, column: 0 }],
        }
    }
}