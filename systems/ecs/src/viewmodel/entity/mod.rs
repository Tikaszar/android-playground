//! Entity management ViewModel functions

mod spawn_entity;
mod despawn_entity;
mod exists;
mod is_alive;
mod clone_entity;

pub use spawn_entity::spawn_entity;
pub use despawn_entity::despawn_entity;
pub use exists::exists;
pub use is_alive::is_alive;
pub use clone_entity::clone_entity;