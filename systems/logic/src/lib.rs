//! Full-featured ECS System for game logic
//! 
//! This layer provides a complete game development framework built on top of core/ecs.
//! It includes hybrid archetype storage, parallel system execution, networked components,
//! and component-based events.

pub mod archetype;
pub mod component;
pub mod entity;
pub mod error;
pub mod event;
pub mod query;
pub mod scheduler;
pub mod storage;
pub mod system;
pub mod world;

pub use archetype::*;
pub use component::*;
pub use entity::*;
pub use error::*;
pub use event::*;
pub use query::*;
pub use scheduler::*;
pub use storage::*;
pub use system::*;
pub use world::*;