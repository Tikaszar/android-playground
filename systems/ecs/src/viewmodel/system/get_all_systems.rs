//! Get all registered systems

use playground_modules_types::{ModuleResult, ModuleError};
use std::pin::Pin;
use std::future::Future;

pub fn get_all_systems(args: &[u8]) -> Pin<Box<dyn Future<Output = ModuleResult<Vec<u8>>> + Send>> {
    let args = args.to_vec();
    Box::pin(async move {
        // TODO: Implement get_all_systems
        Err(ModuleError::NotImplemented("get_all_systems".to_string()))
    })
}
