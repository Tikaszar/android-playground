//! Schedule systems based on dependencies

use playground_modules_types::{ModuleResult, ModuleError};
use std::pin::Pin;
use std::future::Future;

pub fn schedule_systems(args: &[u8]) -> Pin<Box<dyn Future<Output = ModuleResult<Vec<u8>>> + Send>> {
    let args = args.to_vec();
    Box::pin(async move {
        // TODO: Implement schedule_systems
        Err(ModuleError::Generic("schedule_systems".to_string()))
    })
}
