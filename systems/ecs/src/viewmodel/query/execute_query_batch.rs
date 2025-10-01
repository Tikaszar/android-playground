//! Execute a query and return matching entities in batches

use playground_modules_types::{ModuleResult, ModuleError};
use std::pin::Pin;
use std::future::Future;

pub fn execute_query_batch(args: &[u8]) -> Pin<Box<dyn Future<Output = ModuleResult<Vec<u8>>> + Send>> {
    let args = args.to_vec();
    Box::pin(async move {
        // TODO: Implement execute_query_batch
        Err(ModuleError::Generic("execute_query_batch".to_string()))
    })
}
