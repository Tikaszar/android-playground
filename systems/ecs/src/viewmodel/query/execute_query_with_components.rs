//! Execute query and get entities with their components

use playground_modules_types::{ModuleResult, ModuleError};
use std::pin::Pin;
use std::future::Future;

pub fn execute_query_with_components(args: &[u8]) -> Pin<Box<dyn Future<Output = ModuleResult<Vec<u8>>> + Send>> {
    let args = args.to_vec();
    Box::pin(async move {
        // TODO: Implement execute_query_with_components
        Err(ModuleError::NotImplemented("execute_query_with_components".to_string()))
    })
}
