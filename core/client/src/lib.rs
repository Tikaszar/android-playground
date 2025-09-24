//! Core client - ECS components and types for client functionality
//!
//! This package provides client components for the ECS, similar to how
//! core/rendering provides rendering components.
//!
//! Everything is an ECS component - clients, render targets, input state, etc.

// Component modules
pub mod components;
pub mod types;
pub mod input;
pub mod api;

// Re-export all components
pub use components::*;

// Re-export all types
pub use types::*;

// Re-export input types
pub use input::*;

// Re-export API functions
pub use api::*;