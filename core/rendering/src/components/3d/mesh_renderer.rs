//! Mesh renderer component

use serde::{Serialize, Deserialize};
use playground_core_ecs::{EntityIdId, impl_component_data};
use crate::types::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MeshRenderer {
    pub mesh: EntityId,
    pub material: EntityId,
    pub cast_shadows: bool,
    pub receive_shadows: bool,
    pub sort_order: Int,
}

impl_component_data!(MeshRenderer);