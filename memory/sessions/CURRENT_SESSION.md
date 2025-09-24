# Current Session - Active Work

## Session 65: Hot-Loadable Module Architecture Design

### Session Goal
Design and document the complete hot-loadable module system to replace VTable architecture, enabling everything (Core, Systems, Plugins, Apps) to be reloadable at runtime.

### Work Completed This Session

#### 1. Architecture Analysis
- Analyzed current VTable architecture overhead
- Compared with trait/dyn alternatives (rejected due to NO DYN rule)
- Identified performance issues with serialization (~1000ns per call)

#### 2. Module System Design
- Designed complete hot-loadable module architecture using `abi_stable`
- All layers (Core, Systems, Plugins, Apps) become hot-loadable modules
- Zero unsafe code - `abi_stable` handles all FFI
- Direct function calls for hot paths (1-5ns overhead)
- Message bus for cold paths and discovery

#### 3. Dependency Management
- Feature-based dependency system
- Semantic versioning support
- Backwards compatibility mechanisms
- Module declares what it needs (not who needs it)
- Hot-loader computes dependency graph

#### 4. Implementation Planning
- Created migration plan from VTable to modules
- Defined 5-phase implementation approach
- Updated roadmap with concrete milestones
- Sessions 65-70 for module system implementation

#### 5. Documentation Created
- Created `memory/architecture/MODULES.md` - Complete module architecture
- Updated `memory/design/DESIGN.md` - Added module decisions
- Updated `memory/design/ROADMAP.md` - Implementation phases
- Comprehensive hot-reload documentation

### Key Decisions

1. **Replace VTable with Modules** - 1000x performance improvement
2. **Use abi_stable** - Safe FFI without unsafe code
3. **Feature-based dependencies** - Fine-grained version control
4. **Hybrid communication** - Fast path (function pointers) + Slow path (message bus)
5. **Minimal launcher** - Knows nothing about engine
6. **Module management in core/modules** - Not in launcher
7. **MCP integration** - Claude can develop within IDE

### Architecture Highlights

```
launcher (50KB binary)
    ↓ loads
api/ (interfaces)
    ↓ implement
modules/
    ├── core/*     (data/contracts)
    ├── systems/*  (implementations)
    ├── plugins/*  (features)
    └── apps/*     (applications)

All hot-loadable at runtime!
```

### Next Steps (Session 66)
1. Create `api/` crate with module interfaces
2. Add `abi_stable` dependency
3. Create minimal launcher
4. Begin removing VTable from core/ecs
5. Implement first hot-loadable module as proof of concept