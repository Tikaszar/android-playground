//! Entity management ViewModel functions

mod spawn_entity;
mod spawn_batch;
mod spawn_entity_with_id;
mod despawn_entity;
mod despawn_batch;
mod exists;
mod is_alive;
mod clone_entity;
mod get_entity;
mod get_generation;
mod get_all_entities;

pub use spawn_entity::spawn_entity;
pub use spawn_batch::spawn_batch;
pub use spawn_entity_with_id::spawn_entity_with_id;
pub use despawn_entity::despawn_entity;
pub use despawn_batch::despawn_batch;
pub use exists::exists;
pub use is_alive::is_alive;
pub use clone_entity::clone_entity;
pub use get_entity::get_entity;
pub use get_generation::get_generation;
pub use get_all_entities::get_all_entities;