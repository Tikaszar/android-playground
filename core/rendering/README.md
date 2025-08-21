# playground-core-rendering

Core rendering contracts and abstractions for the Android Playground engine.

## Overview

This package defines the fundamental rendering interfaces and data structures used throughout the engine. It provides a platform-agnostic API that can be implemented by different rendering backends (WebGL, Vulkan, etc.).

## Architecture

### Traits

#### `Renderer`
The main rendering interface that all backends must implement:

```rust
#[async_trait]
pub trait Renderer: Send + Sync {
    async fn initialize(&mut self) -> RenderResult<()>;
    async fn begin_frame(&mut self) -> RenderResult<()>;
    async fn execute_commands(&mut self, batch: &RenderCommandBatch) -> RenderResult<()>;
    async fn end_frame(&mut self) -> RenderResult<()>;
    async fn present(&mut self) -> RenderResult<()>;
    async fn resize(&mut self, width: u32, height: u32) -> RenderResult<()>;
    async fn create_render_target(&mut self, width: u32, height: u32) -> RenderResult<Box<dyn RenderTarget>>;
    async fn shutdown(&mut self) -> RenderResult<()>;
    fn capabilities(&self) -> RendererCapabilities;
    fn is_initialized(&self) -> bool;
}
```

#### `RenderTarget`
Represents a surface that can be rendered to (screen, texture, etc.).

#### `CommandEncoder`
Interface for recording render commands (future enhancement).

### Data Structures

#### `RenderCommand`
Enum representing all possible rendering operations:

- `Clear`: Clear with color
- `DrawQuad`: Rectangle with position, size, color
- `DrawText`: Text with position, size, color
- `DrawImage`: Textured quad with UV coordinates
- `DrawLine`: Line segment with width and color
- `DrawCircle`: Circle with fill/stroke options
- `SetClipRect/ClearClipRect`: Clipping regions
- `SetTransform/ResetTransform`: 2D transformations
- `PushState/PopState`: State management

#### `RenderCommandBatch`
Container for commands with frame metadata:

```rust
pub struct RenderCommandBatch {
    commands: Vec<RenderCommand>,
    viewport: Option<Viewport>,
    frame_id: u64,
}
```

#### `RendererCapabilities`
Describes renderer features and limits:

```rust
pub struct RendererCapabilities {
    pub max_texture_size: u32,
    pub max_render_targets: u32,
    pub supports_compute: bool,
    pub supports_instancing: bool,
    pub supports_tessellation: bool,
    pub max_vertex_attributes: u32,
    pub max_uniform_buffer_size: usize,
}
```

## Design Principles

### Simple Data Types
Commands use primitive arrays (`[f32; N]`) instead of complex math types to:
- Reduce dependencies
- Simplify serialization
- Improve WASM compatibility
- Enable network transmission

### Async-First
All operations are async to support:
- Non-blocking I/O
- GPU synchronization
- Resource loading
- Better mobile performance

### Batching
Commands are batched to:
- Reduce API calls
- Improve GPU utilization
- Enable optimizations
- Support frame-based rendering

## Usage

This package is used by:
- **systems/webgl**: WebGL2 implementation
- **systems/ui**: UI rendering system
- **plugins/ui-framework**: Discord-style UI

## Future Enhancements

- [ ] Render passes and subpasses
- [ ] Resource binding sets
- [ ] Compute command support
- [ ] Multi-threaded command recording
- [ ] GPU query support

## Dependencies

- `async-trait`: Async trait definitions
- `serde`: Command serialization
- `thiserror`: Error handling

## License

Part of the Android Playground project.