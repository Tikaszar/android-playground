//! Create a snapshot of current world state

use playground_modules_types::{ModuleResult, ModuleError};
use std::pin::Pin;
use std::future::Future;

pub fn create_snapshot(args: &[u8]) -> Pin<Box<dyn Future<Output = ModuleResult<Vec<u8>>> + Send>> {
    let args = args.to_vec();
    Box::pin(async move {
        // TODO: Implement create_snapshot
        Err(ModuleError::Generic("create_snapshot".to_string()))
    })
}
