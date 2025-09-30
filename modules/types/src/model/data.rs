//! Concrete data structure for models (no traits!)

use serde::{Deserialize, Serialize};

/// Concrete data structure for models in Core modules
/// This replaces the Model trait with a concrete type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelData {
    /// Unique identifier for this model
    pub model_id: String,

    /// Model name
    pub model_name: String,

    /// List of fields this model exposes
    pub fields: Vec<String>,

    /// Raw data storage
    pub data: Vec<u8>,
}