//! Check if any entities match a query

use playground_modules_types::{ModuleResult, ModuleError};
use std::pin::Pin;
use std::future::Future;

pub fn query_has_results(args: &[u8]) -> Pin<Box<dyn Future<Output = ModuleResult<Vec<u8>>> + Send>> {
    let args = args.to_vec();
    Box::pin(async move {
        // TODO: Implement query_has_results
        Err(ModuleError::NotImplemented("query_has_results".to_string()))
    })
}
