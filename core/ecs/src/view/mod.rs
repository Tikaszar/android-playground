//! View - API contract exports

pub mod entity;
pub mod component;
pub mod event;
pub mod query;
pub mod storage;
pub mod system;
pub mod world;

mod r#trait;
mod view;

pub use r#trait::EcsViewTrait;
pub use view::EcsView;

// Re-export all fragment traits
pub use entity::EntityView;
pub use component::ComponentView;
pub use event::EventView;
pub use query::QueryView;
pub use storage::StorageView;
pub use system::SystemView;
pub use world::WorldView;