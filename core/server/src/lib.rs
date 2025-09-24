//! Core server - ECS components and types for server functionality
//!
//! This package provides server components for the ECS, similar to how
//! core/rendering provides rendering components.
//!
//! Everything is an ECS component - servers, connections, channels, etc.

// Component modules
pub mod components;
pub mod types;
pub mod api;

// Re-export all components
pub use components::*;

// Re-export all types
pub use types::*;

// Re-export API functions
pub use api::*;