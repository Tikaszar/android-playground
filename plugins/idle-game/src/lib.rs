mod plugin;
mod state;

use playground_plugin::Plugin;
pub use plugin::IdleGame;
pub use state::{IdleGameState, Generator};

#[no_mangle]
pub extern "C" fn create_plugin() -> Box<dyn Plugin> {
    Box::new(IdleGame::new())
}