//! Main EcsView struct that composes all fragments

use playground_modules_types::{ViewTrait, ViewId, ModelTypeInfo};
use crate::view::{
    EcsViewTrait,
    entity::EntityFragment,
    component::ComponentFragment,
    event::EventFragment,
    query::QueryFragment,
    storage::StorageFragment,
    system::SystemFragment,
    world::WorldFragment,
};

/// The main ECS View struct that composes all fragments
pub struct EcsView {
    entity: EntityFragment,
    component: ComponentFragment,
    event: EventFragment,
    query: QueryFragment,
    storage: StorageFragment,
    system: SystemFragment,
    world: WorldFragment,
}

impl EcsView {
    /// Create a new EcsView with all fragments
    pub const fn new() -> Self {
        Self {
            entity: EntityFragment,
            component: ComponentFragment,
            event: EventFragment,
            query: QueryFragment,
            storage: StorageFragment,
            system: SystemFragment,
            world: WorldFragment,
        }
    }
}

impl ViewTrait for EcsView {
    fn view_id(&self) -> ViewId {
        crate::ECS_VIEW_ID
    }
}

// Implement EcsViewTrait with associated types
impl EcsViewTrait for EcsView {
    type Entity = EntityFragment;
    type Component = ComponentFragment;
    type Event = EventFragment;
    type Query = QueryFragment;
    type Storage = StorageFragment;
    type System = SystemFragment;
    type World = WorldFragment;

    fn entity(&self) -> &Self::Entity {
        &self.entity
    }

    fn component(&self) -> &Self::Component {
        &self.component
    }

    fn event(&self) -> &Self::Event {
        &self.event
    }

    fn query(&self) -> &Self::Query {
        &self.query
    }

    fn storage(&self) -> &Self::Storage {
        &self.storage
    }

    fn system(&self) -> &Self::System {
        &self.system
    }

    fn world(&self) -> &Self::World {
        &self.world
    }
}

/// Static instance for export
static ECS_VIEW: EcsView = EcsView::new();

/// Export for module loader
#[unsafe(no_mangle)]
pub static PLAYGROUND_VIEW: &dyn ViewTrait = &ECS_VIEW;

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