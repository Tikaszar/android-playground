//! Model types for Core modules

pub mod base;
pub mod data;
pub mod type_id;

pub use base::{ModelId, ModelTrait, ModelType, ModelTypeInfo};
pub use data::ModelData;
pub use type_id::model_type_of;
