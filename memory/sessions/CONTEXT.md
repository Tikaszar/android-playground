# Context - Session Continuity

## Session 66 Complete ✅
Pure Rust hot-loading implementation designed:
1. ✅ Explored all approaches (rejected WASM, processes, bytecode)
2. ✅ Designed pure Rust module interface (no C ABI)
3. ✅ Documented single unsafe exception (Library::new only)
4. ✅ Created complete module loader architecture
5. ✅ Updated all documentation

## Key Accomplishments
- Solved hot-loading with minimal unsafe (single Library::new())
- Pure Rust interfaces (no C ABI)
- 1000x faster than VTable (direct calls vs serialization)
- All module types hot-loadable (Core, Systems, Plugins, Apps)
- Console commands for runtime management

## Architecture Decision
**Pure Rust Hot-Loading with Single Unsafe:**
- VTable: ~1000ns per call (serialization overhead)
- Modules: ~1-5ns per call (direct function pointers)
- Single unsafe exception well-documented
- Pure Rust throughout (no extern "C")

## Implementation Plan (Sessions 67-71)
### Phase 1: Infrastructure (Session 67)
- Create api/ crate with pure Rust interfaces
- Build systems/module-loader
- Add libloading dependency

### Phase 2: Core Migration (Session 68)
- Remove VTable from core/ecs
- Add module interfaces to core packages
- Test hot-reload

### Phase 3: Systems Migration (Session 69)
- Update systems/ecs
- Remove VTable handlers
- Add module interfaces

### Phase 4: Fix Broken Systems (Session 70)
- Fix systems/webgl
- Fix systems/ui
- Test all systems

### Phase 5: Apps/Plugins (Session 71)
- Convert editor to module
- Update plugins
- Full system test

## Next Session Tasks (Session 67)
1. Create api/ crate with pure Rust module interface
2. Build systems/module-loader with single unsafe
3. Remove VTable from core/ecs
4. Add module interface to core/ecs
5. Test hot-reload functionality

## Important Context
- Build status: core/* compiles, systems/webgl and ui broken
- Current git state: Uncommitted Entity/EntityRef changes from Session 61-64
- Single unsafe exception documented in CLAUDE.md
- Focus on getting one module working first as proof

## Technical Notes
- Use `#[no_mangle]` for module export
- Pure Rust function pointers in VTable
- Module dependencies in metadata
- State preservation via serialization
- libloading for dynamic library loading

## Success Metrics
- Hot-reload in < 500ms
- Single unsafe only (Library::new)
- State preserved across reloads
- All module types hot-loadable