# Roadmap - Path from Current to Target

## Immediate Priority: Implement Hot-Loadable Module System

### Phase 1: Create Module Infrastructure
1. **Add libloading dependency** to workspace
   - For dynamic library loading
   - Single unsafe exception documented

2. **Create api/ crate** with pure Rust interfaces
   - Module struct (no traits, no dyn)
   - ModuleVTable with pure Rust function pointers
   - ModuleMetadata for dependencies
   - No C ABI or extern "C"

3. **Create systems/module-loader**
   - Single unsafe for Library::new()
   - Schema validation before loading
   - Hot-reload support for all module types

### Phase 2: Migrate Core to Modules
1. **Remove VTable from core packages**
   - Delete vtable.rs files
   - Remove VTable fields from structs
   - Convert delegation to direct calls

2. **Add module interfaces to core/**
   - Each package exports module interface
   - Implement save/restore state
   - Declare feature dependencies

3. **Create core/modules** for management
   - Module creation from templates
   - Build via cargo
   - Load/unload at runtime
   - Dependency tracking

4. **Create core/mcp** for debugging
   - MCP server integration
   - Tool registration
   - Claude development support

### Phase 3: Migrate Systems to Modules
1. **Update systems to use module interface**
   - Remove VTable handlers
   - Implement pure Rust module exports
   - Direct implementation (no delegation)

2. **Optimize module calls**
   - Direct function calls via vtable
   - Serialization for type erasure
   - ~1-5ns call overhead

3. **Remove systems/logic** (deprecated)
   - Already identified as unnecessary
   - Functionality moved elsewhere

### Phase 4: Fix Broken Systems
1. **Fix systems/webgl compilation**
   - Update to query ECS for components
   - Remove singleton access
   - Implement module interface

2. **Fix systems/ui compilation**
   - Complete rewrite for new architecture
   - Remove trait-based code
   - Implement module interface

### Phase 5: Migrate Plugins and Apps
1. **Update all plugins** to use core/* only
   - Remove systems/* dependencies
   - Use module loader for system access
   - Add module interfaces

2. **Convert apps to modules**
   - Editor becomes hot-loadable
   - Game becomes hot-loadable
   - Self-modifying IDE capability

## Next Priority: Core ECS Improvements

### Critical Features
1. **Resource System** - Global data without entities
2. **Component Events** - Change detection
3. **Entity Hierarchy** - Parent/child relationships
4. **Batch Operations** - Bulk entity/component operations
5. **Query Builder** - Better query API

### Important Enhancements
1. **Event System** - Inter-system communication
2. **Component Bundles** - Group common components
3. **System Ordering** - Explicit dependencies
4. **Commands Buffer** - Deferred operations

## Milestones

### Milestone 1: Module System Complete (Session 65-70)
- [ ] All packages use module interface
- [ ] Hot-reload working for everything
- [ ] MCP integration complete
- [ ] Claude can develop within IDE

### Milestone 2: Systems Fixed (Session 71-75)
- [ ] systems/webgl compiles and works
- [ ] systems/ui rewritten and works
- [ ] All systems hot-loadable
- [ ] Performance optimized

### Milestone 3: Plugins Updated (Session 76-80)
- [ ] All plugins use core/* only
- [ ] Plugin hot-reload working
- [ ] IDE plugins functional
- [ ] Game plugins ready

### Milestone 4: Working IDE (Session 81-85)
- [ ] Editor fully functional
- [ ] Self-modifying capability
- [ ] Conversational interface
- [ ] Mobile optimized

### Milestone 5: Game Features (Session 86-90)
- [ ] Basic ECS game working
- [ ] Physics integrated
- [ ] Multiplayer support
- [ ] Asset loading

### Milestone 6: Production Ready (Session 91-100)
- [ ] APK packaging
- [ ] Play Store ready
- [ ] Documentation complete
- [ ] Performance optimized

## Success Criteria

### For Module System
- [ ] Single unsafe only (Library::new)
- [ ] Sub-second hot-reload
- [ ] State preserved across reloads
- [ ] Pure Rust interfaces (no C ABI)

### For Architecture
- [ ] No VTable overhead (direct calls)
- [ ] Clean core/systems separation
- [ ] Everything hot-loadable
- [ ] MCP debugging support

### For Project
- [ ] Runs on Android/Termux
- [ ] 60fps performance
- [ ] < 100MB memory
- [ ] Battery efficient

## Risk Mitigation

### Technical Risks
- **Module compatibility** - Use semantic versioning
- **State preservation** - Comprehensive save/restore
- **Performance overhead** - Fast path for hot operations
- **Build times** - Incremental compilation

### Architecture Risks
- **Dependency cycles** - Detection and prevention
- **Feature explosion** - Careful feature design
- **Version conflicts** - Compatibility checking
- **Module size** - Lazy loading

## Alternative Approaches

### If libloading has issues
- Consider dlopen crate
- Build custom loader
- Accept more unsafe if needed

### If hot-reload too slow
- Pre-compile modules
- Use incremental linking
- Cache built modules

### If state preservation fails
- Add migration system
- Version state formats
- Provide rollback