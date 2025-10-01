//! World management ViewModel functions

mod initialize_world;
mod get_world;
mod shutdown_world;
mod clear_world;
mod get_entity_count;

pub use initialize_world::initialize_world;
pub use get_world::get_world;
pub use shutdown_world::shutdown_world;
pub use clear_world::clear_world;
pub use get_entity_count::get_entity_count;