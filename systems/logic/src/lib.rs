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
pub mod event_data;
pub mod query;
pub mod resource_storage;
pub mod scheduler;
pub mod storage;
pub mod system;
pub mod system_data;
pub mod world;
pub mod systems_manager;
pub mod ui_interface;
pub mod rendering_interface;
pub mod messaging;

pub use archetype::*;
pub use component::*;
pub use entity::*;
pub use error::*;
pub use event::*;
pub use event_data::*;
pub use query::*;
pub use resource_storage::*;
pub use scheduler::*;
pub use storage::*;
pub use system::*;
pub use system_data::*;
pub use world::*;
pub use systems_manager::SystemsManager;
pub use ui_interface::UiInterface;
pub use rendering_interface::{RenderingInterface, RendererWrapper};
pub use messaging::{GameMessageBus, MessageHandlerData, channels};

// Re-export Handle and Shared types for plugins and apps
pub use playground_core_types::{Handle, handle, Shared, shared};

// Re-export core rendering types that plugins need
pub use playground_core_rendering::{RenderCommand, RenderCommandBatch, Viewport};

// Re-export core UI types that plugins need
pub use playground_core_ui::{
    ElementType as UiElementType, 
    Style as UiStyle, 
    Bounds as UiBounds,
    UiCommand, UiEvent, EventResult as UiEventResult,
};