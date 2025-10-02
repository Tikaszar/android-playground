# Current Session - Session 80: Convert Core/Systems to Trait-Based MVVM

## Session Goal
Convert core/ecs and systems/ecs to use the new trait-based MVVM infrastructure completed in Session 79.

## Context from Session 79
Session 79 completed the modules/* infrastructure with:
- Trait-based MVVM (ModelTrait, ViewTrait, ViewModelTrait)
- Triple-nested sharding (ViewId → ModelType → ModelPool)
- Lock-free Views/ViewModels access
- Object recycling in pools
- THE single unsafe block in loader
- All modules/* packages compile ✅

**However**: Core/Systems modules still use OLD Session 78 exports that reference deleted types.

## Work To Do

### 1. Delete Obsolete Files
- `core/ecs/src/module_exports.rs` - References deleted `ViewAPI`
- `systems/ecs/src/module_exports.rs` - References deleted types

### 2. Convert core/ecs/src/view/*.rs to Trait Definitions
Current: Stub async functions that return errors
Target: Trait definitions extending ViewTrait

Example:
```rust
#[async_trait]
pub trait EntityView: ViewTrait {
    async fn spawn_entity(&self, world: &World, components: Vec<Component>) -> EcsResult<Entity>;
    async fn despawn_entity(&self, world: &World, entity: Entity) -> EcsResult<()>;
    // ... all 11 methods
}
```

Files:
- entity.rs (11 methods)
- component.rs (14 methods)
- event.rs (18 methods)
- query.rs (14 methods)
- storage.rs (17 methods)
- system.rs (13 methods)
- world.rs (17 methods)

### 3. Create core/ecs Symbol Exports
Add to core/ecs/src/lib.rs:
```rust
#[no_mangle]
pub static PLAYGROUND_VIEW: &dyn ViewTrait = &EcsView;

#[no_mangle]
pub static PLAYGROUND_MODELS: &[ModelTypeInfo] = &[
    ModelTypeInfo { model_type: 0x0001, type_name: "Entity" },
    ModelTypeInfo { model_type: 0x0002, type_name: "Component" },
    // ... all model types
];
```

### 4. Convert systems/ecs/src/viewmodel/*.rs to Trait Implementations
Current: Old serialization-based implementations
Target: Direct trait implementations

Example:
```rust
pub struct EntityViewModel;

impl ViewTrait for EntityViewModel {
    fn view_id(&self) -> ViewId { 0x1234567890ABCDEF }
}

#[async_trait]
impl ViewModelTrait for EntityViewModel {
    fn view_id(&self) -> ViewId { 0x1234567890ABCDEF }
}

#[async_trait]
impl EntityView for EntityViewModel {
    async fn spawn_entity(&self, world: &World, components: Vec<Component>) -> EcsResult<Entity> {
        // Move implementation from old functions here
    }
}
```

### 5. Create systems/ecs Symbol Export
Add to systems/ecs/src/lib.rs:
```rust
#[no_mangle]
pub static PLAYGROUND_VIEWMODEL: &dyn ViewModelTrait = &EcsViewModel;
```

### 6. Test Compilation
- Verify core/ecs compiles
- Verify systems/ecs compiles
- Verify modules/* still compiles
- Check for any missing dependencies

### 7. Test Module Loading (Stretch Goal)
- Create simple test that loads core/ecs
- Verify symbols extracted correctly
- Verify binding works

## Key Decisions to Make
1. Single EcsView struct or separate structs per domain (Entity, Component, etc.)?
2. ViewId values - use hash of trait name or manual assignment?
3. ModelType values - use hash of type name or manual assignment?

## Success Criteria
- ✅ Obsolete module_exports.rs files deleted
- ✅ All View traits defined in core/ecs
- ✅ All ViewModel implementations in systems/ecs
- ✅ Symbol exports correct
- ✅ Both packages compile
- 🎯 Module loading test passes (stretch)

## Notes
- This is the FIRST real usage of the Session 79 infrastructure
- Will validate the trait-based design works as intended
- May discover issues that need infrastructure fixes
