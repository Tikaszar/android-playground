//! Function signature for View APIs

use crate::ModuleResult;
use std::future::Future;
use std::pin::Pin;

/// Function signature for async View APIs
pub type ViewFunction = fn(args: &[u8]) -> Pin<Box<dyn Future<Output = ModuleResult<Vec<u8>>> + Send>>;