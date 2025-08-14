use playground_plugin::{Plugin, Stateful};
use playground_types::{Context, Event, PluginError, PluginId, PluginMetadata, RenderContext, Version};
use serde::{Deserialize, Serialize};

#[derive(Default, Clone, Serialize, Deserialize)]
struct IdleGameState {
    currency: u64,
    generators: Vec<Generator>,
    multiplier: f64,
}

#[derive(Clone, Serialize, Deserialize)]
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
    fn metadata(&self) -> &PluginMetadata {
        static METADATA: PluginMetadata = PluginMetadata {
            id: PluginId(String::new()),
            name: String::new(),
            version: Version {
                major: 0,
                minor: 1,
                patch: 0,
            },
        };
        &METADATA
    }

    fn on_load(&mut self, _ctx: &mut Context) -> Result<(), PluginError> {
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
    type State = IdleGameState;
    
    fn save_state(&self) -> Self::State {
        self.state.clone()
    }
    
    fn restore_state(&mut self, state: Self::State) {
        self.state = state;
    }
}

#[no_mangle]
pub extern "C" fn create_plugin() -> Box<dyn Plugin> {
    Box::new(IdleGame::new())
}