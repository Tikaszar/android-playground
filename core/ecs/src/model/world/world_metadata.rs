//! World metadata data structure

use serde::{Deserialize, Serialize};

/// World metadata (creation time, last modified, etc)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorldMetadata {
    pub created_at: u64,
    pub last_modified: u64,
    pub version: String,
    pub name: String,
}

impl Default for WorldMetadata {
    fn default() -> Self {
        Self {
            created_at: 0,
            last_modified: 0,
            version: "1.0.0".to_string(),
            name: "default".to_string(),
        }
    }
}
