//! Collection of View API functions

use super::function::ViewFunction;

/// Collection of View API functions
#[derive(Copy, Clone)]
pub struct ViewAPI {
    pub functions: &'static [(&'static str, ViewFunction)],
}