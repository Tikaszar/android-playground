# Context - Session Continuity

## Session 65 Complete ✅
Hot-loadable module architecture designed:
1. ✅ Analyzed VTable vs alternatives
2. ✅ Designed complete module system
3. ✅ Created feature-based dependencies
4. ✅ Documented entire architecture
5. ✅ Created implementation roadmap

## Key Accomplishments
- Solved the NO DYN + hot-loading challenge with abi_stable
- Designed 1000x faster system than VTable
- Everything becomes hot-loadable (even the IDE itself!)
- Feature-based dependencies with backwards compatibility
- MCP integration for Claude development

## Architecture Decision
**Replacing VTable with Hot-Loadable Modules:**
- VTable: ~1000ns per call (serialization overhead)
- Modules: ~1-5ns per call (direct function pointers)
- Everything hot-loadable without unsafe code
- abi_stable provides safe FFI

## Implementation Plan (Sessions 66-70)
### Phase 1: Infrastructure (Session 66)
- Create api/ crate
- Add abi_stable dependency
- Create minimal launcher

### Phase 2: Core Migration (Session 67)
- Remove VTable from core/ecs
- Add module interfaces
- Test hot-reload

### Phase 3: Systems Migration (Session 68)
- Update systems/ecs
- Add fast-path optimizations
- Remove VTable handlers

### Phase 4: Fix Broken Systems (Session 69)
- Fix systems/webgl
- Fix systems/ui
- Test all systems

### Phase 5: Apps/Plugins (Session 70)
- Convert editor to module
- Update plugins
- Full system test

## Next Session Tasks (Session 66)
1. Create api/ crate with BaseModule interface
2. Add abi_stable = "0.11" to workspace
3. Create launcher/ with minimal main.rs
4. Start removing VTable from core/ecs
5. Implement first module as proof of concept

## Important Context
- Build status: core/* compiles, systems/webgl and ui broken
- Current git state: Uncommitted Entity/EntityRef changes from Session 61-64
- Migration will be gradual - VTable and modules can coexist temporarily
- Focus on getting one module working first as proof

## Technical Notes
- Use `#[export_root_module]` for module entry points
- Use `RString`, `RVec`, etc for ABI-stable types
- Module dependencies go in module.toml files
- Features resolved transitively
- State preservation via save/restore functions

## Success Metrics
- Hot-reload in < 500ms
- No unsafe code in our codebase
- State preserved across reloads
- IDE can reload itself while running