# CONTEXT.md - Session Continuity

This file captures the current development session context for seamless continuation in future sessions.

## Last Session Summary

**Date**: 2025-08-16
**Focus**: WebSocket Architecture Documentation & Design Clarification
**Completed**: Full architectural documentation update removing WebRTC, establishing 4-layer system

## Session Achievements

### ✅ Established 4-Layer Architecture

Successfully documented and clarified the complete engine architecture:

1. **Apps Layer** (Top)
   - Complete products (games, IDEs)
   - Manage and coordinate collections of plugins
   - Examples: playground-editor, idle-mmo-rpg

2. **Plugins Layer**
   - Reusable feature modules
   - Use Systems APIs ONLY (never Core directly)
   - Examples: inventory, combat, chat, editor-core
   - Exception: May implement custom Systems internally

3. **Systems Layer**
   - Engine components
   - Use Core APIs ONLY
   - Provide: ui, networking, rendering, physics, logic
   - Register channels 1-999 with core/server

4. **Core Layer** (Foundation)
   - Minimal dependencies
   - Provides: types, server, client, plugin, android
   - WebSocket multiplexer and channel management

### ✅ WebSocket-Only Networking Design

Established complete WebSocket architecture (NO WebRTC):

1. **Binary Protocol**
   - Custom packet structure with channel routing
   - Frame-based batching (configurable, default 60fps)
   - Priority queues per channel (5 levels)
   - Serialization using `bytes` crate

2. **Channel System**
   - Channel 0: Control (registration, discovery)
   - Channels 1-999: Systems
   - Channels 1000+: Plugins/Apps
   - Dynamic runtime registration
   - KV store for discovery

3. **Authentication**
   - Passkey with 1Password integration
   - Google OAuth for external access
   - Server-side 1Password API primary

4. **WASM Compilation Strategies**
   - Separate: Each System, Plugin, App as .wasm
   - Hybrid: Each System and App as .wasm
   - Unified: Complete App as single .wasm

## Technical Design Decisions

### Architectural Rules Established

1. **Strict Layer Separation**
   - Apps → Plugins → Systems → Core
   - Plugins NEVER use Core directly
   - Systems NEVER depend on other Systems
   - All networking through core/server

2. **Server Authority**
   - Browser is purely a view
   - All logic/state on server
   - No client-side decision making

3. **Communication Flow**
   - Plugin → System → core/server → WebSocket → core/client → System → Plugin
   - Frame-based batching always (no immediate sends)
   - Binary protocol for efficiency

### Current Implementation Status

**core/server**: Basic HTTP server, needs WebSocket implementation
**core/client**: Not yet created, will be WASM module
**systems/networking**: Skeleton only, needs full implementation
**systems/ui**: Has misplaced WebSocket code that should use core/server

## Documentation Updates

### Files Modified
- `CLAUDE.md`: Complete rewrite with 4-layer architecture and WebSocket design
- `README.md`: Updated to reflect current architecture (private project)

### Key Changes
- Removed ALL WebRTC references
- Added complete WebSocket protocol documentation
- Established channel architecture
- Defined packet structure and flow
- Added authentication design
- Specified WASM compilation modes

## Next Session Starting Points

### Immediate Implementation Tasks

1. **core/server WebSocket Implementation**
   - Add tokio-tungstenite for WebSocket support
   - Implement channel registration system
   - Create packet routing/multiplexing
   - Add frame-based batching
   - Implement binary serialization

2. **core/client WASM Module**
   - Create new crate for browser client
   - Mirror server channel architecture
   - Implement reconnection logic
   - Add binary message handling

3. **Channel 0 Control Protocol**
   - Registration messages
   - Channel allocation
   - Discovery/query system
   - Error handling

4. **Systems Registration**
   - Update systems/ui to use core/server
   - Update systems/networking implementation
   - Remove direct WebSocket usage from systems

### Architecture Priorities

1. Get basic WebSocket working in core/server
2. Create minimal core/client for testing
3. Implement channel registration
4. Add packet serialization
5. Test with systems/ui terminal

## Important Notes

1. **Local-only focus**: This project runs entirely on Android device
2. **No WebRTC**: All networking is WebSocket-based
3. **Binary protocol**: Using bytes crate for efficiency
4. **Frame batching**: Never send packets immediately
5. **Dynamic channels**: Runtime registration, not compile-time

## Git Status

- Branch: main
- Last commit: "docs: Update architecture to WebSocket-only networking with 4-layer system"
- Documentation fully updated
- Implementation needs to follow

## Session Handoff

The architecture is now fully documented and clarified. The 4-layer system (Apps → Plugins → Systems → Core) is established with strict separation rules. WebSocket-only networking with binary protocol and channel system is designed.

The next session should focus on:
1. Implementing core/server WebSocket multiplexer
2. Creating core/client WASM module
3. Building channel registration system
4. Migrating systems/ui to use core/server
5. Completing systems/networking implementation

All architectural decisions are finalized. Implementation can proceed according to the documented design.