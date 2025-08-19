use crate::state::{EditorState, OpenFile, CursorPosition};
use playground_core_plugin::Plugin;
use playground_core_types::{PluginMetadata, PluginId, Version, Event, Context, PluginError, RenderContext};
use playground_systems_logic::World;
use playground_systems_ui::UiSystem;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::info;
use uuid::Uuid;

pub struct EditorCorePlugin {
    id: Uuid,
    metadata: PluginMetadata,
    state: Arc<RwLock<EditorState>>,
    base_channel: u16,
}

impl EditorCorePlugin {
    pub fn new() -> Self {
        Self {
            id: Uuid::new_v4(),
            metadata: PluginMetadata {
                id: PluginId("editor-core".to_string()),
                name: "Editor Core".to_string(),
                version: Version {
                    major: 0,
                    minor: 1,
                    patch: 0,
                },
            },
            state: Arc::new(RwLock::new(EditorState::default())),
            base_channel: 1000,
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

impl Plugin for EditorCorePlugin {
    fn metadata(&self) -> &PluginMetadata {
        &self.metadata
    }

    fn on_load(&mut self, ctx: &mut Context) -> Result<(), PluginError> {
        info!("Editor Core plugin loaded");
        Ok(())
    }

    fn on_unload(&mut self, ctx: &mut Context) {
        info!("Editor Core plugin unloaded");
    }

    fn update(&mut self, ctx: &mut Context, delta_time: f32) {
        // Update logic here
    }

    fn render(&mut self, ctx: &mut RenderContext) {
        // Render logic here
    }

    fn on_event(&mut self, event: &Event) -> bool {
        // Handle events
        false
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