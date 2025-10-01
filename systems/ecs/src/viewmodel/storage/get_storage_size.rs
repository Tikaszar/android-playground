//! Get storage size in bytes

use playground_modules_types::{ModuleResult, ModuleError};
use std::pin::Pin;
use std::future::Future;

pub fn get_storage_size(args: &[u8]) -> Pin<Box<dyn Future<Output = ModuleResult<Vec<u8>>> + Send>> {
    let args = args.to_vec();
    Box::pin(async move {
        // TODO: Implement get_storage_size
        Err(ModuleError::NotImplemented("get_storage_size".to_string()))
    })
}
