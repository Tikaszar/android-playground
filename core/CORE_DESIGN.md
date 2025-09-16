# Core Extensibility Design

This document outlines the final design for extending engine capabilities in a way that provides compile-time safety for the API surface while allowing runtime flexibility in choosing implementations.

The design is a two-phase model that covers both compile-time contract definition and runtime implementation exposure.

## Phase 1: Compile-Time (Defining the API Surface)

This phase uses Cargo features and a build script to build a tiered API in `systems/logic` based on the capabilities of all bundled systems.

1.  **Generic Feature Contracts in `core`**:
    -   To add a new capability (e.g., "instancing"), the relevant `core` package (e.g., `core/renderer`) is modified.
    -   A new, optional feature is added to the `core` package's `Cargo.toml` (e.g., `instancing = []`).
    -   A new, feature-gated file is added to the `core` package containing the generic contract, which is a trait (e.g., `#[cfg(feature = "instancing")] pub trait InstancedRendering { ... }`).
    -   Alongside the trait, a concrete "VTable" struct is defined (e.g., `InstancedRenderingVTable`), which will hold function pointers and act as a resource in the ECS.
    -   This is the acceptable pattern for extending `core`: adding new, generic, feature-gated contracts.

2.  **Systems Implement Contracts**:
    -   A `system` (e.g., `systems/webgl`) that supports this feature enables it on its `core` dependency in its own `Cargo.toml` (e.g., `playground-core-renderer = {..., features = ["instancing"] }`).
    -   The `system` then provides the specialized implementation for the generic contract trait (e.g., `impl InstancedRendering for WebGLSystem { ... }`).

3.  **Tiered API Generation in `systems/logic`**:
    -   `systems/logic` has a `build.rs` build script that acts as a compile-time auditor.
    -   The script inspects the application's final dependency graph and finds all systems of a given type (e.g., all renderers).
    -   It computes the **intersection** of features (supported by all) and the **optional features** (supported by some, but not all).
    -   It generates a namespaced API in `systems/logic` based on these two sets:
        -   **Guaranteed API:** For features in the intersection, it generates functions with direct return types (e.g., `-> InstancingResult`). These are guaranteed to be safe at runtime.
        -   **Optional API:** For optional features, it generates functions that return a `Result` (e.g., `-> Result<MeshResult, CoreError>`). This provides a compile-time check that the feature is *possible* while forcing the developer to handle runtime failure.

## Phase 2: Runtime (Activating an Implementation)

This phase uses the `core/ecs` `World` as a discovery and dispatch hub for the active implementations.

1.  **Application Selects System**:
    -   The `app` reads its runtime configuration file (e.g., `config.toml`) at startup to decide which concrete `system` to use (e.g., `"webgl"`).
    -   It then initializes only the selected `system` (e.g., `WebGLSystem`).

2.  **Runtime Exposure via ECS**:
    -   The initialized `WebGLSystem` is responsible for "exposing" its implementation of any feature contracts it supports.
    -   It does this by building the corresponding `VTable` struct (e.g., `InstancedRenderingVTable`), populating it with function pointers that point to its own methods.
    -   It then registers this `VTable` struct as a **singleton resource** within the `core/ecs` `World`.

3.  **Dispatch from `systems/logic`**:
    -   When a plugin calls a function from the generated API (e.g., `logic::renderer::instancing::draw_instanced()`), that function's implementation in `logic` is simple:
        1.  Access the `World`.
        2.  Request the corresponding `VTable` singleton resource (e.g., `InstancingVTable`).
        3.  Call the appropriate function pointer from the VTable struct.
    -   For the Guaranteed API, this is guaranteed to succeed. For the Optional API, this query might fail (if the active system doesn't support the feature), which is handled by the `Result` return type.
