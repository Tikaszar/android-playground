//! Restore world from a snapshot

use playground_modules_types::{ModuleResult, ModuleError};
use std::pin::Pin;
use std::future::Future;

pub fn restore_snapshot(args: &[u8]) -> Pin<Box<dyn Future<Output = ModuleResult<Vec<u8>>> + Send>> {
    let args = args.to_vec();
    Box::pin(async move {
        // TODO: Implement restore_snapshot
        Err(ModuleError::Generic("restore_snapshot".to_string()))
    })
}
