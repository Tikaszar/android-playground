//! Save specific entities to storage

use playground_modules_types::{ModuleResult, ModuleError};
use std::pin::Pin;
use std::future::Future;

pub fn save_entities(args: &[u8]) -> Pin<Box<dyn Future<Output = ModuleResult<Vec<u8>>> + Send>> {
    let args = args.to_vec();
    Box::pin(async move {
        // TODO: Implement save_entities
        Err(ModuleError::NotImplemented("save_entities".to_string()))
    })
}
