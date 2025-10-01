//! Entity management ViewModel functions

mod spawn_entity;
mod spawn_batch;
mod despawn_entity;
mod despawn_batch;
mod exists;
mod is_alive;
mod clone_entity;

pub use spawn_entity::spawn_entity;
pub use spawn_batch::spawn_batch;
pub use despawn_entity::despawn_entity;
pub use despawn_batch::despawn_batch;
pub use exists::exists;
pub use is_alive::is_alive;
pub use clone_entity::clone_entity;