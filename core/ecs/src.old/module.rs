//! Module interface for core/ecs
//!
//! This exports the module interface for hot-loading support.
//! Core modules are data-only, so this is primarily for metadata.

use playground_api::*;

// Module export - PLAYGROUND_MODULE is the symbol that will be loaded
playground_api::define_module! {
    name: "core/ecs",
    version: "1.0.0",
    type: ModuleType::Core,
    dependencies: [],
    features: [
        "entities",
        "components",
        "queries",
        "messaging",
        "systems",
        "storage",
        "registry"
    ],
    vtable: CORE_ECS_VTABLE
}

/// Core modules are data-only, so we use the no-op vtable
static CORE_ECS_VTABLE: ModuleVTable = NOOP_VTABLE;