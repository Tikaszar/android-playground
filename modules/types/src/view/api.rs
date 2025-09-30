//! Collection of View API functions

use super::function::ViewFunction;

/// Collection of View API functions
pub struct ViewAPI {
    pub functions: &'static [(&'static str, ViewFunction)],
}