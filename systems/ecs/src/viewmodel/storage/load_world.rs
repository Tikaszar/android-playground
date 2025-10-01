//! Load the entire world from storage

use playground_modules_types::{ModuleResult, ModuleError};
use std::pin::Pin;
use std::future::Future;

pub fn load_world(args: &[u8]) -> Pin<Box<dyn Future<Output = ModuleResult<Vec<u8>>> + Send>> {
    let args = args.to_vec();
    Box::pin(async move {
        // TODO: Implement load_world
        Err(ModuleError::NotImplemented("load_world".to_string()))
    })
}
