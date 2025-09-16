//! Error types for the ECS
//! 
//! Re-exports error types from core/types with ECS-specific helpers.

use crate::{EntityId, ComponentId};

// Re-export from core/types
pub use playground_core_types::{CoreError, CoreResult, EntityIdError, ComponentIdError};

// Helper functions for ECS-specific errors
/// Create an EntityNotFound error from an EntityId
pub fn entity_not_found(entity: EntityId) -> CoreError {
    CoreError::EntityNotFound(EntityIdError(entity.index()))
}

/// Create a ComponentNotFound error
pub fn component_not_found(entity: EntityId, component_id: ComponentId) -> CoreError {
    CoreError::ComponentNotFound(EntityIdError(entity.index()), component_id.0)
}

// Keep compatibility aliases
pub type EcsError = CoreError;
pub type EcsResult<T> = CoreResult<T>;