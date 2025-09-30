//! Type of module determining its role in MVVM

use serde::{Deserialize, Serialize};

/// Type of module determining its role in MVVM
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ModuleType {
    /// Core module providing Model + View
    Core,

    /// System module providing ViewModel
    System,

    /// Plugin module using Core APIs
    Plugin,

    /// App module orchestrating everything
    App,
}