# Context - Session Continuity

## Current Task
Fixing systems/networking to comply with architecture:
1. Remove unsafe usage (CRITICAL)
2. Implement client operations
3. Remove non-networking handlers

## Where We Left Off
- Created memory/* organization structure
- Analyzed systems/networking violations
- Developed fix plan based on systems/console pattern
- Ready to implement fixes

## Key Understanding
- Apps/Plugins use core/* ONLY (with features)
- Systems/* use core/* ONLY (implement contracts)
- Systems cannot import other systems
- VTable provides cross-boundary communication

## Next Immediate Steps
1. Replace `static mut` with `OnceCell` in vtable_handlers.rs
2. Access Server/Client via core API functions
3. Implement client WebSocket operations
4. Remove rendering/audio/input handlers
5. Test compilation

## Important Context
- Last commit "Partial Work, Needs some fixes" has unsafe violations
- systems/console shows correct OnceCell pattern to follow
- NetworkState should only store implementation details
- Server/Client instances exist globally in core packages

## Session Goals
- [ ] Fix unsafe usage
- [ ] Complete client implementation
- [ ] Remove wrong operations
- [ ] Get clean compilation
- [ ] Update CLAUDE.md with memory references

## Notes for Next Session
The memory/* structure is now in place. Continue with the actual code fixes to systems/networking following the patterns established in memory/architecture/PATTERNS.md.