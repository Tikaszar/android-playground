# Roadmap - Path from Current to Target

## Immediate Priority: Fix systems/networking

### Task 1: Remove unsafe usage ⚠️ CRITICAL
- Replace `static mut` with `OnceCell`
- Use pattern from systems/console
- Store only implementation state, not Server/Client instances
- Access core instances through API functions

### Task 2: Complete client implementation
- Implement WebSocket client operations
- Use existing WebSocketHandler
- Store client connections properly
- Update core/client state fields

### Task 3: Remove non-networking operations
- Remove rendering handlers (belong in systems/webgl)
- Remove audio handlers (belong in systems/audio)
- Remove input handlers (belong in systems/input)
- Only handle networking operations

### Task 4: Test VTable integration
- Verify server operations work
- Test client connections
- Check channel operations
- Validate error handling

## Next Priority: Fix systems/webgl

### Task 1: Implement client VTable handlers
- Register with core/client
- Handle rendering operations
- Update render targets
- Manage WebGL resources

### Task 2: Remove direct dependencies
- No imports from other systems
- Use core/client data fields
- Communicate via VTable/ECS

## Architecture Fixes Needed

### Systems Layer
1. **systems/ui** - Complete rewrite needed
   - Remove all trait-based code
   - Implement VTable handlers
   - Use core/ui contracts

2. **systems/logic** - Complete rewrite needed
   - No longer API gateway
   - Should be game logic system
   - Or remove entirely

3. **systems/physics** - Complete rewrite needed
   - Implement core/physics contracts
   - 2D physics first
   - Mobile-optimized

### Plugins Layer
All 9 IDE plugins need rewrite to use core/* only:
- chat-assistant
- debugger
- editor-core
- file-browser
- lsp-client
- terminal
- theme-manager
- ui-framework
- version-control

## Milestones

### Milestone 1: Core Compliance ✅ COMPLETE
- All core packages follow data vs logic pattern
- VTable dispatch working
- Feature flags implemented

### Milestone 2: Systems Compliance (IN PROGRESS)
- [ ] systems/networking fixed
- [ ] systems/webgl fixed
- [ ] systems/ui rewritten
- [ ] systems/logic removed or rewritten
- [ ] systems/physics implemented

### Milestone 3: Plugin Compliance
- [ ] All plugins use core/* only
- [ ] No direct system dependencies
- [ ] Feature flags used properly

### Milestone 4: Working IDE
- [ ] Editor functional
- [ ] Terminal working
- [ ] File browser operational
- [ ] Build and run from IDE

### Milestone 5: Game Features
- [ ] Basic ECS game
- [ ] Physics integration
- [ ] Multiplayer support
- [ ] Asset loading

### Milestone 6: Production Ready
- [ ] APK packaging
- [ ] Play Store ready
- [ ] Documentation complete
- [ ] Performance optimized

## Dependencies

```
Core Compliance (✅)
    ↓
Systems Compliance
    ├── systems/networking (current)
    ├── systems/webgl
    ├── systems/console (✅)
    └── systems/ecs (✅)
    ↓
Plugin Compliance
    ├── IDE plugins
    └── Game plugins
    ↓
Working IDE
    ↓
Game Features
    ↓
Production
```

## Success Criteria

### For Current Task (systems/networking)
- [ ] NO unsafe code
- [ ] Client operations implemented
- [ ] Only networking operations
- [ ] All tests passing

### For Architecture
- [ ] All violations fixed
- [ ] Compile-time error detection
- [ ] Zero runtime failures
- [ ] Clean separation of concerns

### For Project
- [ ] Runs on Android/Termux
- [ ] 60fps performance
- [ ] < 100MB memory
- [ ] Battery efficient

## Timeline Estimates

### Short Term (1-2 sessions)
- Fix systems/networking
- Fix systems/webgl
- Basic testing

### Medium Term (5-10 sessions)
- Rewrite remaining systems
- Update all plugins
- IDE functional

### Long Term (20+ sessions)
- Game features
- Performance optimization
- Production packaging
- Documentation

## Risk Factors

### Technical Risks
- Plugin rewrites may be complex
- Performance on older devices
- Battery usage optimization
- Network latency handling

### Architecture Risks
- System isolation may limit features
- VTable overhead
- Feature flag complexity
- Compile time growth

## Alternative Approaches

### If VTable overhead too high
- Direct function pointers
- Compile-time dispatch
- Macro-based routing

### If compile times too long
- Reduce feature flags
- Split into smaller crates
- Use workspace optimization

### If memory usage too high
- More aggressive cleanup
- Smaller buffers
- Lazy initialization
- Resource pooling