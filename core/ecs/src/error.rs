//! Error types for the ECS module

use playground_core_types::{CoreError, CoreResult};

// Re-export with ECS-specific aliases
pub type EcsError = CoreError;
pub type EcsResult<T> = CoreResult<T>;