//! Module export symbols for dynamic loading

use playground_modules_types::{
    Module, ModuleMetadata, ModuleType, ModuleLifecycle, ViewModelImpl, ModuleDependency,
};

// Module dependencies
static DEPENDENCIES: &[ModuleDependency] = &[
    ModuleDependency {
        name: "playground-core-ecs",
        version_req: "1.0.0",
        features: &[],
    },
];

// Module metadata
static METADATA: ModuleMetadata = ModuleMetadata {
    name: "systems-ecs",
    version: "1.0.0",
    description: "ECS ViewModel implementation",
    features: &[], // No optional features
    dependencies: DEPENDENCIES,
};

// Lifecycle functions
static LIFECYCLE: ModuleLifecycle = ModuleLifecycle {
    initialize: module_initialize,
    shutdown: module_shutdown,
    save_state: module_save_state,
    restore_state: module_restore_state,
};

// Module export for hot-loading
#[unsafe(no_mangle)]
pub static PLAYGROUND_MODULE: Module = Module {
    metadata: &METADATA,
    module_type: ModuleType::System,
    lifecycle: LIFECYCLE,
};

// ViewModel implementation export for System modules
#[unsafe(no_mangle)]
pub static PLAYGROUND_VIEWMODEL_IMPL: ViewModelImpl = ViewModelImpl {
    view_id: "core-ecs", // Matches the Core module we implement
    functions: &[
        // World functions (5)
        ("initialize_world", crate::viewmodel::world::initialize_world),
        ("get_world", crate::viewmodel::world::get_world),
        ("shutdown_world", crate::viewmodel::world::shutdown_world),
        ("clear_world", crate::viewmodel::world::clear_world),
        ("get_entity_count", crate::viewmodel::world::get_entity_count),

        // Entity functions (5)
        ("spawn_entity", crate::viewmodel::entity::spawn_entity),
        ("despawn_entity", crate::viewmodel::entity::despawn_entity),
        ("exists", crate::viewmodel::entity::exists),
        ("is_alive", crate::viewmodel::entity::is_alive),
        ("clone_entity", crate::viewmodel::entity::clone_entity),

        // Component functions (5)
        ("add_component", crate::viewmodel::component::add_component),
        ("remove_component", crate::viewmodel::component::remove_component),
        ("get_component", crate::viewmodel::component::get_component),
        ("has_component", crate::viewmodel::component::has_component),
        ("get_all_components", crate::viewmodel::component::get_all_components),

        // Event functions (4)
        ("publish_pre_event", crate::viewmodel::event::publish_pre_event),
        ("publish_post_event", crate::viewmodel::event::publish_post_event),
        ("subscribe_event", crate::viewmodel::event::subscribe_event),
        ("unsubscribe_event", crate::viewmodel::event::unsubscribe_event),
    ],
};

// Lifecycle implementations
fn module_initialize(_args: &[u8]) -> Result<(), String> {
    // World gets initialized on first use
    Ok(())
}

fn module_shutdown() -> Result<(), String> {
    // World gets cleared in shutdown_world
    Ok(())
}

fn module_save_state() -> Vec<u8> {
    // For hot-reload - would serialize world state
    // For now, return empty
    vec![]
}

fn module_restore_state(_state: &[u8]) -> Result<(), String> {
    // For hot-reload - would restore world state
    Ok(())
}