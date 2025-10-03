//! Runtime type ID generation for Models

use std::any::TypeId;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use super::ModelType;

/// Generate a unique ModelType from a Rust type at runtime
///
/// This function generates a deterministic ModelType value based on
/// the TypeId of the given type. The same type will always produce
/// the same ModelType value, but the value is generated at runtime
/// rather than being hardcoded.
///
/// Benefits:
/// - No manual constants that could overlap
/// - Compile-time type safety (wrong types won't compile)
/// - Deterministic (same type always generates same ModelType)
/// - Clean (no enums or constants to maintain)
pub fn model_type_of<T: 'static>() -> ModelType {
    let type_id = TypeId::of::<T>();
    let mut hasher = DefaultHasher::new();
    type_id.hash(&mut hasher);
    hasher.finish()
}