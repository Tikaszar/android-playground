//! Function signature for ViewModel implementations

use crate::ModuleResult;
use std::future::Future;
use std::pin::Pin;

/// Function signature for ViewModel implementations
pub type ViewModelFunction = fn(args: &[u8]) -> Pin<Box<dyn Future<Output = ModuleResult<Vec<u8>>> + Send>>;