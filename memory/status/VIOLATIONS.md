# Violations - Current Architecture Violations

## Critical Violations 🔴

### 1. ~~core/rendering singleton pattern~~ ✅ FIXED
**Location**: core/rendering
**Status**: RESOLVED in Session 64
**Solution Applied**:
- Deleted renderer.rs and operations.rs
- Removed singleton RENDERER_INSTANCE
- Removed separate VTable
- Everything is now entities with components
- API functions take EntityRef parameters

### 2. ~~unsafe usage in systems/networking~~ ✅ FIXED
**Location**: systems/networking/src/vtable_handlers.rs
**Status**: RESOLVED in Session 58
**Solution Applied**:
- Replaced `static mut` with `Lazy<NetworkState>`
- Used `Handle<T>` and `Shared<T>` type aliases consistently
- Removed all unsafe blocks

### 2. ~~core/server and core/client singletons~~ ✅ FIXED
**Location**: core/server and core/client
**Status**: RESOLVED in Session 62
**Solution Applied**:
- Complete rewrite to use ECS components
- Removed singleton Server and Client structs
- Everything is now entities with components
- API functions return Entity handles

### 3. Plugins don't compile
**Location**: All plugins/* packages
**Issue**: Dependencies removed but code still imports systems/*
```rust
// VIOLATION: All plugin source files still have:
use playground_systems_logic::...
use playground_systems_ui::...
```
**Fix Required**: Complete rewrite to use core/* only with feature flags
**Priority**: CRITICAL - Nothing compiles

## Major Violations 🟠

### 4. ~~systems/networking needs updating for new ECS~~ ✅ FIXED
**Location**: systems/networking
**Status**: RESOLVED in Session 63
**Solution Applied**:
- Complete rewrite to use ECS entities
- Created state management for Entity references
- VTable registration through world.vtable
- All compilation errors fixed

### 5. systems/webgl needs updating for new ECS
**Location**: systems/webgl
**Issue**: Still expects old singleton patterns, needs to query ECS
**Fix Required**: Update to query for rendering components
**Priority**: HIGH - Blocks rendering

### 6. systems/ui needs complete rewrite
**Location**: systems/ui
**Issue**: Old architecture, uses traits instead of VTable
**Fix Required**: Complete rewrite following data vs logic pattern
**Priority**: HIGH - Blocks UI

### 6. ~~systems/logic deprecated~~ ✅ REMOVED
**Location**: systems/logic
**Status**: REMOVED in Session 59
**Solution Applied**:
- Deleted systems/logic directory
- Removed from workspace
- Removed all dependencies

## Minor Violations 🟡

### 6. Incomplete client implementation
**Location**: systems/networking/src/vtable_handlers.rs:466-529
**Issue**: Client operations mostly stubbed
```rust
async fn handle_client_send(_payload: Bytes) -> VTableResponse {
    // Send data through WebSocket
    // Implementation would send through active connection
    success_response(None)
}
```
**Fix Required**: Implement actual WebSocket client
**Priority**: HIGH - Needed for testing

### 7. ~~Non-networking operations in networking~~ ✅ FIXED
**Location**: systems/networking/src/vtable_handlers.rs
**Status**: RESOLVED in Session 58
**Solution Applied**:
- Removed `handle_render_operations` (belongs in systems/webgl)
- Removed `handle_audio_operations` (belongs in systems/audio)
- Removed `handle_input_operations` (belongs in systems/input)
- Updated registration.rs to remove VTable registrations

### 8. Error handling inconsistent
**Location**: Various vtable_handlers
**Issue**: Some operations return generic errors or silently succeed
**Fix Required**: Consistent error messages with context
**Priority**: LOW - Quality issue

## Code Smell Issues 💭

### 9. NetworkServer duplicates Server fields
**Location**: systems/networking/src/server.rs
**Issue**: Some fields duplicate what's in core/server
**Analysis**: This is actually OK - NetworkServer holds implementation-specific state
**Action**: No fix needed, but document better

### 10. Global state management
**Location**: Various systems
**Issue**: Different patterns for managing global state
**Fix Required**: Standardize on OnceCell pattern
**Priority**: LOW - Consistency issue

## Documentation Violations 📝

### 11. Outdated architecture docs
**Location**: README.md, old ROADMAP.md
**Issue**: Still reference old architecture
**Fix Required**: Update to reflect current architecture
**Priority**: LOW - Documentation debt

## Summary by Component

| Component | Critical | Major | Minor | Total |
|-----------|----------|-------|-------|-------|
| systems/networking | 0 ✅ | 0 ✅ | 0 ✅ | 0 ✅ |
| systems/webgl | 0 | 1 | 0 | 1 |
| systems/ui | 0 | 1 | 0 | 1 |
| plugins/* | 1 | 0 | 0 | 1 |
| Documentation | 0 | 0 | 1 | 1 |

**Total**: 1 Critical (plugins), 2 Major (webgl, ui), 1 Minor violations

## Fix Order

1. **First**: Fix systems/webgl compilation (Blocks everything)
2. **Second**: Fix systems/ui compilation (Blocks plugins)
3. **Third**: Rewrite all plugins to use core/* (Major effort)
4. **Fourth**: Complete client implementation (For testing)
5. **Later**: Documentation updates