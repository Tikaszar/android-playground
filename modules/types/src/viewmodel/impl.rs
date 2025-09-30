//! Collection of ViewModel implementation functions

use super::function::ViewModelFunction;

/// Collection of ViewModel implementation functions
pub struct ViewModelImpl {
    pub view_id: &'static str,
    pub functions: &'static [(&'static str, ViewModelFunction)],
}