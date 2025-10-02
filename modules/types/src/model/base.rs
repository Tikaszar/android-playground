//! Model trait for data structures (Core modules only)
//! Pure data with no logic, stored in module registry pools

/// 64-bit unique identifier for a model instance
pub type ModelId = u64;

/// 64-bit unique identifier for a model type (for pool routing)
pub type ModelType = u64;

/// Model trait for pure data structures with no logic
///
/// Core modules define Models that contain:
/// - Data fields
/// - State structures
/// - Configuration types
/// - NO implementation logic
///
/// Models are stored in pools within the module registry
#[async_trait::async_trait]
pub trait ModelTrait: Send + Sync {
    /// Get the unique instance ID of this model
    fn model_id(&self) -> ModelId;

    /// Get the type ID for pool routing
    fn model_type(&self) -> ModelType;
}

/// Information about a model type for pool initialization
#[derive(Debug, Clone, Copy)]
pub struct ModelTypeInfo {
    /// The model type ID (for pool routing)
    pub model_type: ModelType,

    /// Human-readable type name (for debugging)
    pub type_name: &'static str,
}
