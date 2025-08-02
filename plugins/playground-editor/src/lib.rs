use playground_plugin::{Plugin, Stateful};
use playground_types::{Context, Event, RenderContext};
use serde::{Deserialize, Serialize};
use std::error::Error;

#[derive(Default, Serialize, Deserialize)]
struct EditorState {
    current_file: Option<String>,
    cursor_position: (usize, usize),
    selection: Option<(usize, usize)>,
    open_files: Vec<String>,
}

pub struct PlaygroundEditor {
    state: EditorState,
}

impl PlaygroundEditor {
    pub fn new() -> Self {
        Self {
            state: EditorState::default(),
        }
    }
}

impl Plugin for PlaygroundEditor {
    fn id(&self) -> &str {
        "playground-editor"
    }

    fn name(&self) -> &str {
        "Playground Editor"
    }

    fn version(&self) -> &str {
        "0.1.0"
    }

    fn dependencies(&self) -> Vec<&str> {
        vec!["ui", "networking"]
    }

    fn on_load(&mut self, _ctx: &mut Context) -> Result<(), Box<dyn Error>> {
        tracing::info!("Playground Editor plugin loaded");
        Ok(())
    }

    fn on_unload(&mut self, _ctx: &mut Context) {
        tracing::info!("Playground Editor plugin unloading");
    }

    fn update(&mut self, _ctx: &mut Context, _delta_time: f32) {
        // Handle editor updates
    }

    fn render(&mut self, _ctx: &mut RenderContext) {
        // Render editor UI
    }

    fn on_event(&mut self, event: &Event) -> bool {
        match event.id.as_str() {
            "file_open" => {
                // Handle file open
                true
            }
            "file_save" => {
                // Handle file save
                true
            }
            _ => false,
        }
    }
}

impl Stateful for PlaygroundEditor {
    fn save_state(&self) -> serde_json::Value {
        serde_json::to_value(&self.state).unwrap_or(serde_json::Value::Null)
    }

    fn load_state(&mut self, state: serde_json::Value) {
        if let Ok(loaded_state) = serde_json::from_value(state) {
            self.state = loaded_state;
        }
    }
}

#[no_mangle]
pub extern "C" fn create_plugin() -> Box<dyn Plugin> {
    Box::new(PlaygroundEditor::new())
}