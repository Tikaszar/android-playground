//! Collection of ViewModel implementation functions

use crate::viewmodel_function::ViewModelFunction;

/// Collection of ViewModel implementation functions
pub struct ViewModelImpl {
    pub view_id: &'static str,
    pub functions: Vec<(&'static str, ViewModelFunction)>,
}