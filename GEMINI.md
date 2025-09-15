# GEMINI.md

This file provides guidance to Gemini when working with code in this repository.

## Project Overview

Android Playground is a mobile-first, plugin-based game engine designed for development entirely on Android devices using Termux. The architecture prioritizes hot-reload capabilities, battery efficiency, and a conversational-first IDE.

## Architecture: The 4-Layer Model

The project follows a strict 4-layer architecture. The guiding principles are:

1.  **`core` is for Generic Primitives Only**: The `core` layer must only define contracts for universal, application-agnostic primitives (ECS, messaging, rendering).
2.  **`systems` are Private Implementations**: Systems crates are concrete, private implementations of the `core` contracts. They must not expose a public API and should only be interacted with via the command processors defined in `core`.
3.  **`systems/logic` is the Sole Public API**: This is the only crate that exposes a public API for the engine.
4.  **`plugins` are Consumers of the Public API**: Plugins must *only* depend on `systems/logic` and interact with the engine exclusively through its public API.

### The Unified ECS Architecture

The engine uses a single, unified ECS. The roles of the key crates are:

*   **`core/ecs`**: Defines the contracts for the ECS (e.g., `WorldContract`, `ComponentData`, `System` traits) but contains **no implementation**.
*   **`systems/ecs`**: Contains the **single, authoritative implementation** of the `World` and all related ECS mechanics (storage, scheduler, etc.).
*   **`systems/logic`**: A **stateless API gateway**. Plugins and Apps interact *only* with this crate. It forwards calls to the appropriate `core` command processors, hiding all engine internals.

### Plugin Architecture: Plugins are Systems

The old `core/plugin` crate has been removed. The current architecture treats plugins as high-level systems.

*   Plugins implement the `systems/logic::System` trait.
*   They are loaded and managed by the `App` (e.g., `playground-editor`).
*   They interact with the engine exclusively through the `systems/logic` API.
*   They must not depend on any other `systems/*` or `core/*` crates.

## Current Status (Post-Audit - 2025-09-15)

A comprehensive architectural audit of the `core`, `systems`, and `plugins` layers is complete. **The project is not in a state to add new features.** Major refactoring is required to bring the implementation in line with the architecture.

*   **`core` Layer:** ✅ **Conceptually Sound.** The design is complete and correct. Minor implementation bugs (`dyn`/`Any` usage) and one misplaced module (`core/android`) need to be fixed.

*   **`systems` Layer:** ❌ **NOT ALIGNED.** This layer has significant architectural problems.
    *   **Require Rewrite:** `systems/ui`, `systems/logic`, `systems/physics`.
    *   **Require Refactor:** `systems/networking`, `systems/ecs`, `systems/webgl`.

*   **`plugins` Layer:** ❌ **Fundamentally Broken.**
    *   **All 9 IDE plugins are non-compliant** and require a complete rewrite. They bypass the `systems/logic` API gateway and depend directly on other systems.

## Next Steps

The immediate and only priority is to fix the foundational architectural violations identified in the audit, starting with the `systems` layer. New feature development is on hold until the codebase is compliant.

1.  **Refactor `systems/ui`** to remove its dependency on `systems/networking`.
2.  **Rewrite `systems/logic`** to be a pure, stateless API gateway that does not depend on other systems.
3.  **Fix `dyn` violations** in `systems/ecs` and `systems/networking`.
4.  **Address all other `systems` layer issues.**
5.  **Rewrite all `plugins`** to use only the `systems/logic` API.

## Key Design Decisions to Enforce

1.  **Strict Layer Separation**: `App` → `Plugin` → `systems/logic` → `core/*` / `systems/*`.
2.  **System Isolation**: `systems` crates must not depend on each other.
3.  **NO `dyn` / `NO Any`**: All trait objects are forbidden. Use concrete wrappers and command objects.
    *   *Violation found in `core/types/context.rs` and `core/server/*` that must be fixed.*
4.  **Server-Side Authority**: The browser is a pure view; all logic is on the server.
5.  **Private Systems**: `systems` crates should not expose a public API. All interaction must be through the command processor pattern defined in `core`.

## UI System Design Note

The audit revealed that `systems/ui` may contain a conflicting `InternalElementStorage` that deviates from the principle of using the main ECS for all state management. This needs to be addressed during its rewrite.