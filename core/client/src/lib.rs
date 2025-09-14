//! Generic client contracts
//!
//! This module defines abstract contracts for ANY client implementation
//! (browser/WebGL, native/Vulkan, mobile app, CLI tool, etc.)

mod contracts;
mod types;
mod input;
mod commands;

pub use contracts::*;
pub use types::*;
pub use input::*;
pub use commands::*;