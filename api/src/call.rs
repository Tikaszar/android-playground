//! Standard module interface for communication

use serde::{Serialize, Deserialize};

/// Standard module call structure
#[derive(Debug, Serialize, Deserialize)]
pub struct ModuleCall {
    /// Source module making the call
    pub source: String,
    /// Target module to call
    pub target: String,
    /// Method to invoke
    pub method: String,
    /// Serialized arguments
    pub args: Vec<u8>,
}

/// Standard module response structure
#[derive(Debug, Serialize, Deserialize)]
pub struct ModuleResponse {
    /// Whether the call succeeded
    pub success: bool,
    /// Serialized result data
    pub data: Option<Vec<u8>>,
    /// Error message if failed
    pub error: Option<String>,
}