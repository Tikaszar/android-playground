//! Mesh resource component

#[cfg(all(feature = "core-3d", feature = "buffers"))]
use serde::{Serialize, Deserialize};
#[cfg(all(feature = "core-3d", feature = "buffers"))]
use playground_core_ecs::impl_component_data;
#[cfg(all(feature = "core-3d", feature = "buffers"))]
use crate::types::*;

#[cfg(all(feature = "core-3d", feature = "buffers"))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Mesh {
    pub gpu_resource_id: ResourceId,
    pub vertex_count: UInt,
    pub index_count: UInt,
    pub bounds: BoundingBox,
}

#[cfg(all(feature = "core-3d", feature = "buffers"))]
impl_component_data!(Mesh);