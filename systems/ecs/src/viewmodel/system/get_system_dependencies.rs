//! Get system dependencies

use playground_modules_types::{ModuleResult, ModuleError};
use std::pin::Pin;
use std::future::Future;

pub fn get_system_dependencies(args: &[u8]) -> Pin<Box<dyn Future<Output = ModuleResult<Vec<u8>>> + Send>> {
    let args = args.to_vec();
    Box::pin(async move {
        // TODO: Implement get_system_dependencies
        Err(ModuleError::Generic("get_system_dependencies".to_string()))
    })
}
