//! Core ECS - Foundation for the entire engine
//! 
//! This provides the fundamental ECS infrastructure that all other packages build upon.
//! Apps and Plugins import this PLUS the specific core/* packages they need.
//! 
//! Example usage:
//! ```rust
//! use playground_core_ecs::{World, Entity, Component, initialize_world};
//! use playground_core_server::api as networking;  // Additional import for networking
//! use playground_core_ui::api as ui;              // Additional import for UI
//! ```

// Core ECS modules (always available)
pub mod entity;
pub mod entity_ref;
pub mod generation;
pub mod component;
pub mod error;
pub mod world;
pub mod vtable;
pub mod registry;
pub mod messaging;
pub mod query;
pub mod storage;
pub mod system;

// Re-export core ECS types (always available)
pub use entity::*;
pub use entity_ref::{Entity, EntityRef};
pub use generation::Generation;
pub use component::*;
pub use error::*;
pub use world::*;
pub use vtable::*;
pub use registry::*;
pub use messaging::*;
pub use query::*;
pub use storage::*;
pub use system::*;