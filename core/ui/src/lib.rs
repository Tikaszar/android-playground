//! Core UI contracts and abstractions
//! 
//! This package defines the base traits and types for UI systems.
//! It provides contracts that concrete UI implementations (like systems/ui)
//! must implement, enabling pluggable UI backends.

pub mod traits;
pub mod types;
pub mod commands;
pub mod events;
pub mod error;

pub use traits::*;
pub use types::*;
pub use commands::*;
pub use events::*;
pub use error::*;