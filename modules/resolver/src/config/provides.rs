//! What a System module provides

use serde::{Deserialize, Serialize};

/// What a System module provides
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemProvides {
    /// Which Core module this implements
    pub core: String,

    /// Features this System provides
    pub features: Vec<String>,
}