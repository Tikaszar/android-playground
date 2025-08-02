use playground_plugin::{Plugin, Stateful};
use playground_types::{Context, Event, RenderContext};
use serde::{Deserialize, Serialize};
use std::error::Error;

#[derive(Default, Serialize, Deserialize)]
struct IdleGameState {
    currency: u64,
    generators: Vec<Generator>,
    multiplier: f64,
}

#[derive(Serialize, Deserialize)]
struct Generator {
    name: String,
    count: u32,
    base_cost: u64,
    production: f64,
}

pub struct IdleGame {
    state: IdleGameState,
}

impl IdleGame {
    pub fn new() -> Self {
        Self {
            state: IdleGameState::default(),
        }
    }
}

impl Plugin for IdleGame {
    fn id(&self) -> &str {
        "idle-game"
    }

    fn name(&self) -> &str {
        "Idle Game"
    }

    fn version(&self) -> &str {
        "0.1.0"
    }

    fn dependencies(&self) -> Vec<&str> {
        vec!["ui", "logic"]
    }

    fn on_load(&mut self, _ctx: &mut Context) -> Result<(), Box<dyn Error>> {
        tracing::info!("Idle Game plugin loaded");
        Ok(())
    }

    fn on_unload(&mut self, _ctx: &mut Context) {
        tracing::info!("Idle Game plugin unloading");
    }

    fn update(&mut self, _ctx: &mut Context, delta_time: f32) {
        for generator in &self.state.generators {
            self.state.currency += 
                (generator.count as f64 * generator.production * delta_time as f64 * self.state.multiplier) as u64;
        }
    }

    fn render(&mut self, _ctx: &mut RenderContext) {
        // Render game UI
    }

    fn on_event(&mut self, _event: &Event) -> bool {
        false
    }
}

impl Stateful for IdleGame {
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
    Box::new(IdleGame::new())
}