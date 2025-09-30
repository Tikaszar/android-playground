//! Model base class for data structures (Core modules only)
//! Using concrete base class instead of trait (NO dyn allowed!)

use serde::{Deserialize, Serialize};

/// Model base class for pure data structures with no logic
///
/// Core modules define Models that contain:
/// - Data fields
/// - State structures
/// - Configuration types
/// - NO implementation logic
///
/// This is a concrete base class, not a trait!
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Model {
    /// Unique identifier for this model type
    pub model_id: &'static str,

    /// Model name
    pub model_name: &'static str,

    /// List of fields this model exposes
    pub fields: Vec<&'static str>,

    /// Serialized data (for type erasure without Any)
    pub data: Vec<u8>,
}