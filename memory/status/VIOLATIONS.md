# Violations - Current Architecture Violations

## Critical Violations 🔴

### 1. ~~unsafe usage in systems/networking~~ ✅ FIXED
**Location**: systems/networking/src/vtable_handlers.rs
**Status**: RESOLVED in Session 58
**Solution Applied**:
- Replaced `static mut` with `Lazy<NetworkState>`
- Used `Handle<T>` and `Shared<T>` type aliases consistently
- Removed all unsafe blocks

### 2. Plugins bypass core architecture
**Location**: All plugins/* packages
**Issue**: Import systems/ui, systems/networking directly instead of using core/*
```rust
// VIOLATION in plugins/*/Cargo.toml
playground-systems-ui = { path = "../../systems/ui" }
```
**Fix Required**: Rewrite to use core/* only
**Priority**: HIGH - Breaks layering

## Major Violations 🟠

### 3. systems/webgl missing VTable handlers
**Location**: systems/webgl
**Issue**: Doesn't implement core/client rendering operations
**Fix Required**: Add VTable handler registration like systems/networking
**Priority**: HIGH - Blocks rendering

### 4. systems/ui needs complete rewrite
**Location**: systems/ui
**Issue**: Old architecture, uses traits instead of VTable
**Fix Required**: Complete rewrite following data vs logic pattern
**Priority**: HIGH - Blocks UI

### 5. systems/logic architecture unclear
**Location**: systems/logic
**Issue**: Was API gateway, now core/* serves that purpose
**Fix Required**: Either remove or repurpose for game logic
**Priority**: MEDIUM - Architectural debt

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
| systems/networking | ~~1~~ 0 ✅ | 0 | ~~3~~ 1 | ~~4~~ 1 |
| systems/webgl | 0 | 1 | 0 | 1 |
| systems/ui | 0 | 1 | 0 | 1 |
| systems/logic | 0 | 1 | 0 | 1 |
| plugins/* | 1 | 0 | 0 | 1 |
| Documentation | 0 | 0 | 1 | 1 |

**Total**: ~~2~~ 1 Critical, 3 Major, ~~4~~ 2 Minor violations

## Fix Order

1. **First**: ~~Remove unsafe from systems/networking~~ ✅ COMPLETE
2. **Second**: Complete client implementation (Blocks testing)
3. **Third**: ~~Remove non-networking operations~~ ✅ COMPLETE
4. **Fourth**: Implement systems/webgl VTable (Blocks rendering)
5. **Later**: Rewrite systems/ui, all plugins, documentation