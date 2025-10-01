# Current Session - Session 74: Implement systems/ecs ViewModel Layer

## Session Goal
Complete the ViewModel implementation for systems/ecs to match all View API contracts from core/ecs.

## Current Status

### Audit Completed ✅
Compared core/ecs/view API contracts with systems/ecs/viewmodel implementations:
- **View API contracts**: 101 functions across 7 modules
- **ViewModel implementations started**: 58 functions
- **Missing implementations**: 48 functions

### Started Implementations (58 functions) ✅
**Entity (5)**: spawn_entity, despawn_entity, exists, is_alive, clone_entity
**Component (5)**: add_component, remove_component, get_component, has_component, get_all_components
**Event (4)**: publish_pre_event, publish_post_event, subscribe_event, unsubscribe_event
**Query (14)**: create_query, execute_query, execute_query_batch, query_count, delete_query, update_query, get_query, get_all_queries, query_has_results, query_first, execute_query_with_components, query_entities, query_exists, clone_query
**Storage (17)**: create_storage, delete_storage, get_storage, get_all_storages, storage_exists, get_storage_size, clear_storage, save_world, load_world, save_entities, load_entities, create_snapshot, restore_snapshot, delete_snapshot, list_snapshots, export_json, import_json
**System (12)**: register_system, unregister_system, run_system, run_systems, schedule_systems, step_systems, enable_system, disable_system, is_system_enabled, get_system, get_all_systems, get_system_stats, get_system_dependencies
**World (6)**: initialize_world, get_world, shutdown_world, clear_world, get_entity_count

### Missing Implementations (48 functions)

**Priority 1 - Core Entity/Component (14 functions):**
- Entity: spawn_batch, despawn_batch, spawn_entity_with_id, get_entity, get_generation, get_all_entities
- Component: add_components, remove_components, get_components, has_components, clear_components, count_components, replace_component, get_entities_with_component, get_entities_with_components

**Priority 2 - Event System (14 functions):**
- subscribe_pre, subscribe_post, unsubscribe, unsubscribe_all
- emit_event, emit_batch, emit_event_with_priority
- process_event_queue, process_high_priority_events, clear_event_queue
- get_event_queue_size, get_pending_events, get_subscription, get_subscriptions, has_subscribers

**Priority 3 - System Management (4 functions):**
- system_exists, get_dependent_systems, update_system_dependencies, clear_system_stats

**Priority 4 - World Operations (11 functions):**
- get_stats, get_world_metadata, clone_world, merge_worlds
- lock_world, unlock_world, is_world_locked, reset_world, validate_world, step

**Priority 5 - Resource Operations (5 functions - stub for now):**
- insert_resource, get_resource, remove_resource, has_resource, get_all_resources
- Note: World Model doesn't include resources storage, will stub with "not supported" errors

## Work Completed This Session

### 1. Initial Audit ✅
- Listed all View API functions (101 total)
- Listed all ViewModel implementations (58 started)
- Identified 48 missing implementations
- Updated viewmodel/mod.rs to export query, storage, system modules

### 2. Architecture Review ✅
- Confirmed ViewModel pattern: functions take `&[u8]`, return `Pin<Box<dyn Future<Output = ModuleResult<Vec<u8>>>>>`
- Confirmed World Model structure (no resources field - will stub those functions)
- Confirmed module_exports.rs pattern for function registration

### 3. Component Module Complete ✅ (14/14 functions)
- Implemented 9 missing functions:
  - add_components, remove_components (batch operations)
  - get_components, has_components (multi-query operations)
  - clear_components, count_components (utility operations)
  - replace_component (upsert operation)
  - get_entities_with_component, get_entities_with_components (search operations)
- Fixed lifetime issues in existing functions by copying args before async blocks
- Updated mod.rs exports

### 4. Entity Module Progress ✅ (7/11 functions)
- Implemented 2 missing functions:
  - spawn_batch, despawn_batch (batch operations)
- Fixed lifetime issues in existing functions (clone, despawn, exists, is_alive)
- Updated mod.rs exports
- **Remaining**: get_entity, get_generation, get_all_entities, spawn_entity_with_id

### 5. Core Model Improvements ✅
- Added serde::Serialize and serde::Deserialize derives to Component struct
- Enabled serde feature for bytes crate in workspace Cargo.toml
- Entity serialization strategy: use (EntityId, Generation) tuples instead of full Entity struct

## Implementation Pattern Established
```rust
pub fn function_name(args: &[u8]) -> Pin<Box<dyn Future<Output = ModuleResult<Vec<u8>>> + Send>> {
    let args = args.to_vec();  // Copy args before async block (lifetime fix)
    Box::pin(async move {
        let args: ArgsStruct = bincode::deserialize(&args)?;
        let world = crate::state::get_world()?;
        // ... implementation ...
        Ok(bincode::serialize(&result)?)
    })
}
```

## Next Steps
1. Complete remaining 4 entity functions (get_entity, get_generation, get_all_entities, spawn_entity_with_id)
2. Fix event module (serde_bytes issues)
3. Fix query/storage/system stub errors (ModuleError::NotImplemented → Generic)
4. Implement remaining Priority 2-5 functions
5. Update module_exports.rs with all function registrations
6. Test full compilation
