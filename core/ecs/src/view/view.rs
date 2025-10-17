//! Main EcsView struct that composes all fragments

use playground_modules_types::{ViewTrait, ViewId, ModelTypeInfo, model_type_of};
use crate::{
    model::{Entity, Component, Event, Query, Storage, System},
    view::{
        EcsViewTrait,
        entity::EntityFragment,
        component::ComponentFragment,
        event::EventFragment,
        query::QueryFragment,
        storage::StorageFragment,
        system::SystemFragment,
        world::WorldFragment,
    },
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

    fn api_version(&self) -> u32 {
        crate::API_VERSION
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

/// Generate model type info at runtime
/// This will be called by the module loader to get the model types
pub fn get_model_types() -> Vec<ModelTypeInfo> {
    vec![
        ModelTypeInfo {
            model_type: model_type_of::<Entity>(),
            type_name: "Entity"
        },
        ModelTypeInfo {
            model_type: model_type_of::<Component>(),
            type_name: "Component"
        },
        ModelTypeInfo {
            model_type: model_type_of::<Event>(),
            type_name: "Event"
        },
        ModelTypeInfo {
            model_type: model_type_of::<Query>(),
            type_name: "Query"
        },
        ModelTypeInfo {
            model_type: model_type_of::<Storage>(),
            type_name: "Storage"
        },
        ModelTypeInfo {
            model_type: model_type_of::<System>(),
            type_name: "System"
        },
    ]
}

/// Model type information for pool initialization
/// Note: This is empty at compile-time and filled at runtime by the loader
#[unsafe(no_mangle)]
pub static PLAYGROUND_MODELS: &[ModelTypeInfo] = &[];