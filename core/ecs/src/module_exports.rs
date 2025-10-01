//! Module export symbols for dynamic loading

use playground_modules_types::{
    Module, ModuleMetadata, ModuleType, ModuleLifecycle, ViewAPI, ModuleResult, ModuleError,
};
use std::pin::Pin;
use std::future::Future;

// Module metadata
static METADATA: ModuleMetadata = ModuleMetadata {
    name: "core-ecs",
    version: "1.0.0",
    description: "Core ECS with Event System",
    features: &[],  // No optional features - everything is fundamental
    dependencies: &[],
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
    module_type: ModuleType::Core,
    lifecycle: LIFECYCLE,
};

// View API export for Core modules
#[unsafe(no_mangle)]
pub static PLAYGROUND_VIEW_API: ViewAPI = ViewAPI {
    functions: &[
        // World functions
        ("initialize_world", view_initialize_world),
        ("get_world", view_get_world),
        ("shutdown_world", view_shutdown_world),
        ("clear_world", view_clear_world),
        ("get_entity_count", view_get_entity_count),

        // Entity functions
        ("spawn_entity", view_spawn_entity),
        ("despawn_entity", view_despawn_entity),
        ("exists", view_exists),
        ("is_alive", view_is_alive),
        ("clone_entity", view_clone_entity),

        // Component functions
        ("add_component", view_add_component),
        ("remove_component", view_remove_component),
        ("get_component", view_get_component),
        ("has_component", view_has_component),
        ("get_all_components", view_get_all_components),

        // Event functions
        ("publish_pre_event", view_publish_pre_event),
        ("publish_post_event", view_publish_post_event),
        ("subscribe_event", view_subscribe_event),
        ("unsubscribe_event", view_unsubscribe_event),
    ],
};

// View function wrappers that match ViewFunction signature
fn view_initialize_world(_args: &[u8]) -> Pin<Box<dyn Future<Output = ModuleResult<Vec<u8>>> + Send>> {
    Box::pin(async move {
        // This would be implemented by systems/ecs
        Err(ModuleError::Generic("initialize_world not implemented in core".to_string()))
    })
}

fn view_get_world(_args: &[u8]) -> Pin<Box<dyn Future<Output = ModuleResult<Vec<u8>>> + Send>> {
    Box::pin(async move {
        Err(ModuleError::Generic("get_world not implemented in core".to_string()))
    })
}

fn view_shutdown_world(_args: &[u8]) -> Pin<Box<dyn Future<Output = ModuleResult<Vec<u8>>> + Send>> {
    Box::pin(async move {
        Err(ModuleError::Generic("shutdown_world not implemented in core".to_string()))
    })
}

fn view_clear_world(_args: &[u8]) -> Pin<Box<dyn Future<Output = ModuleResult<Vec<u8>>> + Send>> {
    Box::pin(async move {
        Err(ModuleError::Generic("clear_world not implemented in core".to_string()))
    })
}

fn view_get_entity_count(_args: &[u8]) -> Pin<Box<dyn Future<Output = ModuleResult<Vec<u8>>> + Send>> {
    Box::pin(async move {
        Err(ModuleError::Generic("get_entity_count not implemented in core".to_string()))
    })
}

fn view_spawn_entity(_args: &[u8]) -> Pin<Box<dyn Future<Output = ModuleResult<Vec<u8>>> + Send>> {
    Box::pin(async move {
        Err(ModuleError::Generic("spawn_entity not implemented in core".to_string()))
    })
}

fn view_despawn_entity(_args: &[u8]) -> Pin<Box<dyn Future<Output = ModuleResult<Vec<u8>>> + Send>> {
    Box::pin(async move {
        Err(ModuleError::Generic("despawn_entity not implemented in core".to_string()))
    })
}

fn view_exists(_args: &[u8]) -> Pin<Box<dyn Future<Output = ModuleResult<Vec<u8>>> + Send>> {
    Box::pin(async move {
        Err(ModuleError::Generic("exists not implemented in core".to_string()))
    })
}

fn view_is_alive(_args: &[u8]) -> Pin<Box<dyn Future<Output = ModuleResult<Vec<u8>>> + Send>> {
    Box::pin(async move {
        Err(ModuleError::Generic("is_alive not implemented in core".to_string()))
    })
}

fn view_clone_entity(_args: &[u8]) -> Pin<Box<dyn Future<Output = ModuleResult<Vec<u8>>> + Send>> {
    Box::pin(async move {
        Err(ModuleError::Generic("clone_entity not implemented in core".to_string()))
    })
}

fn view_add_component(_args: &[u8]) -> Pin<Box<dyn Future<Output = ModuleResult<Vec<u8>>> + Send>> {
    Box::pin(async move {
        Err(ModuleError::Generic("add_component not implemented in core".to_string()))
    })
}

fn view_remove_component(_args: &[u8]) -> Pin<Box<dyn Future<Output = ModuleResult<Vec<u8>>> + Send>> {
    Box::pin(async move {
        Err(ModuleError::Generic("remove_component not implemented in core".to_string()))
    })
}

fn view_get_component(_args: &[u8]) -> Pin<Box<dyn Future<Output = ModuleResult<Vec<u8>>> + Send>> {
    Box::pin(async move {
        Err(ModuleError::Generic("get_component not implemented in core".to_string()))
    })
}

fn view_has_component(_args: &[u8]) -> Pin<Box<dyn Future<Output = ModuleResult<Vec<u8>>> + Send>> {
    Box::pin(async move {
        Err(ModuleError::Generic("has_component not implemented in core".to_string()))
    })
}

fn view_get_all_components(_args: &[u8]) -> Pin<Box<dyn Future<Output = ModuleResult<Vec<u8>>> + Send>> {
    Box::pin(async move {
        Err(ModuleError::Generic("get_all_components not implemented in core".to_string()))
    })
}

fn view_publish_pre_event(_args: &[u8]) -> Pin<Box<dyn Future<Output = ModuleResult<Vec<u8>>> + Send>> {
    Box::pin(async move {
        Err(ModuleError::Generic("publish_pre_event not implemented in core".to_string()))
    })
}

fn view_publish_post_event(_args: &[u8]) -> Pin<Box<dyn Future<Output = ModuleResult<Vec<u8>>> + Send>> {
    Box::pin(async move {
        Err(ModuleError::Generic("publish_post_event not implemented in core".to_string()))
    })
}

fn view_subscribe_event(_args: &[u8]) -> Pin<Box<dyn Future<Output = ModuleResult<Vec<u8>>> + Send>> {
    Box::pin(async move {
        Err(ModuleError::Generic("subscribe_event not implemented in core".to_string()))
    })
}

fn view_unsubscribe_event(_args: &[u8]) -> Pin<Box<dyn Future<Output = ModuleResult<Vec<u8>>> + Send>> {
    Box::pin(async move {
        Err(ModuleError::Generic("unsubscribe_event not implemented in core".to_string()))
    })
}

// Lifecycle implementations
fn module_initialize(_args: &[u8]) -> Result<(), String> {
    Ok(())
}

fn module_shutdown() -> Result<(), String> {
    Ok(())
}

fn module_save_state() -> Vec<u8> {
    // For hot-reload - would serialize world state
    vec![]
}

fn module_restore_state(_state: &[u8]) -> Result<(), String> {
    // For hot-reload - would restore world state
    Ok(())
}