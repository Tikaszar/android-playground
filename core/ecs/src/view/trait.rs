//! EcsViewTrait - ensures all fragments are implemented

use crate::view::{
    EntityView, ComponentView, EventView, QueryView,
    StorageView, SystemView, WorldView
};

/// ECS View trait that requires all fragment implementations
pub trait EcsViewTrait:
    EntityView +
    ComponentView +
    EventView +
    QueryView +
    StorageView +
    SystemView +
    WorldView
{}