//! EcsViewTrait - ensures all fragments are implemented

use playground_modules_types::ViewTrait;
use crate::view::{
    EntityView, ComponentView, EventView, QueryView,
    StorageView, SystemView, WorldView
};

/// ECS View trait that requires all fragment implementations via associated types
pub trait EcsViewTrait: ViewTrait {
    /// Entity operations fragment
    type Entity: EntityView;

    /// Component operations fragment
    type Component: ComponentView;

    /// Event operations fragment
    type Event: EventView;

    /// Query operations fragment
    type Query: QueryView;

    /// Storage operations fragment
    type Storage: StorageView;

    /// System operations fragment
    type System: SystemView;

    /// World operations fragment
    type World: WorldView;

    /// Get the entity fragment
    fn entity(&self) -> &Self::Entity;

    /// Get the component fragment
    fn component(&self) -> &Self::Component;

    /// Get the event fragment
    fn event(&self) -> &Self::Event;

    /// Get the query fragment
    fn query(&self) -> &Self::Query;

    /// Get the storage fragment
    fn storage(&self) -> &Self::Storage;

    /// Get the system fragment
    fn system(&self) -> &Self::System;

    /// Get the world fragment
    fn world(&self) -> &Self::World;
}