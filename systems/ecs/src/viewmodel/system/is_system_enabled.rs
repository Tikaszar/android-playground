//! Check if a system is enabled

use playground_modules_types::{ModuleResult, ModuleError};
use std::pin::Pin;
use std::future::Future;

pub fn is_system_enabled(args: &[u8]) -> Pin<Box<dyn Future<Output = ModuleResult<Vec<u8>>> + Send>> {
    let args = args.to_vec();
    Box::pin(async move {
        // TODO: Implement is_system_enabled
        Err(ModuleError::NotImplemented("is_system_enabled".to_string()))
    })
}
