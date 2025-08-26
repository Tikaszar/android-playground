use async_trait::async_trait;
use crate::state::{EditorState, OpenFile, CursorPosition};
use playground_systems_logic::{System, World, LogicResult, SystemsManager};
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{info, debug};

pub struct EditorCorePlugin {
    state: Arc<RwLock<EditorState>>,
    base_channel: u16,
    systems_manager: Arc<SystemsManager>,
}

impl EditorCorePlugin {
    pub fn new(systems_manager: Arc<SystemsManager>) -> Self {
        Self {
            state: Arc::new(RwLock::new(EditorState::default())),
            base_channel: 1000,
            systems_manager,
        }
    }

    pub async fn open_file(&self, path: String, content: String) {
        let mut state = self.state.write().await;
        
        if let Some(index) = state.open_files.iter().position(|f| f.path == path) {
            state.active_file = Some(index);
            return;
        }
        
        let language = Self::detect_language(&path);
        
        state.open_files.push(OpenFile {
            path,
            content,
            language,
            modified: false,
        });
        
        state.active_file = Some(state.open_files.len() - 1);
    }

    pub async fn close_file(&self, path: &str) {
        let mut state = self.state.write().await;
        
        if let Some(index) = state.open_files.iter().position(|f| f.path == path) {
            state.open_files.remove(index);
            
            if let Some(active) = state.active_file {
                if active >= state.open_files.len() && !state.open_files.is_empty() {
                    state.active_file = Some(state.open_files.len() - 1);
                } else if state.open_files.is_empty() {
                    state.active_file = None;
                }
            }
        }
    }

    pub async fn toggle_vim_mode(&self) {
        let mut state = self.state.write().await;
        state.vim_mode = !state.vim_mode;
        info!("Vim mode: {}", state.vim_mode);
    }

    pub async fn add_cursor(&self, line: usize, column: usize) {
        let mut state = self.state.write().await;
        state.cursors.push(CursorPosition { line, column });
    }

    pub async fn clear_cursors(&self) {
        let mut state = self.state.write().await;
        if !state.cursors.is_empty() {
            let primary = state.cursors[0].clone();
            state.cursors = vec![primary];
        }
    }

    fn detect_language(path: &str) -> String {
        match path.split('.').last() {
            Some("rs") => "rust".to_string(),
            Some("js") => "javascript".to_string(),
            Some("ts") => "typescript".to_string(),
            Some("py") => "python".to_string(),
            Some("go") => "go".to_string(),
            Some("java") => "java".to_string(),
            Some("cpp") | Some("cc") | Some("cxx") => "cpp".to_string(),
            Some("c") | Some("h") => "c".to_string(),
            Some("md") => "markdown".to_string(),
            Some("json") => "json".to_string(),
            Some("toml") => "toml".to_string(),
            Some("yaml") | Some("yml") => "yaml".to_string(),
            _ => "text".to_string(),
        }
    }
}


// State management methods
impl EditorCorePlugin {
    pub async fn save_state(&self) -> EditorState {
        self.state.read().await.clone()
    }

    pub fn load_state(&mut self, state: EditorState) {
        self.state = Arc::new(RwLock::new(state));
    }
}

#[async_trait]
impl System for EditorCorePlugin {
    fn name(&self) -> &'static str {
        "EditorCorePlugin"
    }
    
    async fn initialize(&mut self, _world: &World) -> LogicResult<()> {
        info!("Editor Core Plugin initializing on channel {}", self.base_channel);
        
        
        // Initialize default editor state
        let mut state = self.state.write().await;
        state.vim_mode = true;  // Enable vim mode by default
        
        Ok(())
    }
    
    async fn run(&mut self, _world: &World, _delta_time: f32) -> LogicResult<()> {
        // Process any pending editor operations
        // This would handle vim commands, cursor movements, etc.
        Ok(())
    }
    
    async fn cleanup(&mut self, _world: &World) -> LogicResult<()> {
        info!("Editor Core Plugin shutting down");
        
        // Save any unsaved changes
        let state = self.state.read().await;
        for file in &state.open_files {
            if file.modified {
                debug!("Warning: Unsaved changes in {}", file.path);
            }
        }
        
        Ok(())
    }
}