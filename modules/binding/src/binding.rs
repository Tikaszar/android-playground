//! A bound View-ViewModel pair

use playground_modules_types::ViewModelFunction;
use std::collections::HashMap;

/// A bound View-ViewModel pair enabling direct function calls
pub struct Binding {
    /// The View ID this binding is for
    pub view_id: String,

    /// Direct function pointers to ViewModel implementations
    pub functions: HashMap<String, ViewModelFunction>,
}