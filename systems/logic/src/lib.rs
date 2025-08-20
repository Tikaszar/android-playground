//! Full-featured ECS System for game logic
//! 
//! This layer provides a complete game development framework built on top of core/ecs.
//! It includes hybrid archetype storage, parallel system execution, networked components,
//! and component-based events.
//!
//! Systems/logic is responsible for initializing ALL other systems in the engine.

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
pub mod systems_manager;

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
pub use systems_manager::SystemsManager;

// Re-export Shared type for plugins and apps
pub use playground_core_types::{Shared, shared};