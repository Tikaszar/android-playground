//! Export world to JSON format

use playground_modules_types::{ModuleResult, ModuleError};
use std::pin::Pin;
use std::future::Future;

pub fn export_json(args: &[u8]) -> Pin<Box<dyn Future<Output = ModuleResult<Vec<u8>>> + Send>> {
    let args = args.to_vec();
    Box::pin(async move {
        // TODO: Implement export_json
        Err(ModuleError::NotImplemented("export_json".to_string()))
    })
}
