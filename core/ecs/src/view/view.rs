//! Main EcsView struct that implements all fragments

use playground_modules_types::{ViewTrait, ViewId, ModelTypeInfo};
use crate::view::EcsViewTrait;

/// The main ECS View struct
pub struct EcsView;

impl ViewTrait for EcsView {
    fn view_id(&self) -> ViewId {
        crate::ECS_VIEW_ID
    }
}

// Mark that EcsView implements all fragments
// The actual implementations are in each fragment's view.rs file
impl EcsViewTrait for EcsView {}

/// Export for module loader
#[unsafe(no_mangle)]
pub static PLAYGROUND_VIEW: &dyn ViewTrait = &EcsView;

/// Model type information for pool initialization
#[unsafe(no_mangle)]
pub static PLAYGROUND_MODELS: &[ModelTypeInfo] = &[
    ModelTypeInfo { model_type: 0x0001, type_name: "Entity" },
    ModelTypeInfo { model_type: 0x0002, type_name: "Component" },
    ModelTypeInfo { model_type: 0x0003, type_name: "Event" },
    ModelTypeInfo { model_type: 0x0004, type_name: "Query" },
    ModelTypeInfo { model_type: 0x0005, type_name: "Storage" },
    ModelTypeInfo { model_type: 0x0006, type_name: "System" },
    ModelTypeInfo { model_type: 0x0007, type_name: "World" },
];