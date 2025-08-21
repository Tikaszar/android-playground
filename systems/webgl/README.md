# playground-systems-webgl

WebGL2 renderer implementation for the Android Playground engine.

## Overview

This package provides a complete WebGL2-based renderer that implements the `core/rendering::Renderer` trait. It's designed for high-performance 2D rendering in web browsers, with a focus on mobile efficiency.

## Features

- **WebGL2 Support**: Modern GPU features including instancing
- **Command Batching**: Reduces draw calls by batching geometry
- **Transform Stack**: Hierarchical 2D transformations with Matrix3
- **Clip Rectangles**: Scissor-based clipping regions
- **State Management**: Push/pop render states
- **Texture Management**: Efficient texture caching and binding
- **Shader System**: Customizable vertex/fragment shaders

## Architecture

### Components

- `WebGLRenderer`: Main renderer implementing core traits
- `WebGLContext`: Low-level WebGL API wrapper
- `VertexBuffer/IndexBuffer`: Geometry batching system
- `ShaderProgram`: Shader compilation and management
- `Texture2D`: Texture loading and binding

### Render Commands Supported

- `Clear`: Clear screen with color
- `DrawQuad`: Filled rectangles with color/texture
- `DrawText`: Simple text rendering
- `DrawImage`: Textured quad with UV mapping
- `DrawLine`: Lines with configurable width
- `DrawCircle`: Filled or outlined circles
- `SetClipRect/ClearClipRect`: Scissor regions
- `SetTransform/ResetTransform`: 2D transformations
- `PushState/PopState`: State save/restore

## Usage

```rust
use playground_systems_webgl::WebGLRenderer;
use playground_core_rendering::{Renderer, RenderCommandBatch};

let mut renderer = WebGLRenderer::new();
renderer.initialize().await?;

let mut batch = RenderCommandBatch::new(frame_id);
batch.push(RenderCommand::Clear { 
    color: [0.1, 0.1, 0.1, 1.0] 
});
batch.push(RenderCommand::DrawQuad {
    position: [100.0, 100.0],
    size: [200.0, 150.0],
    color: [1.0, 0.0, 0.0, 1.0],
});

renderer.begin_frame().await?;
renderer.execute_commands(&batch).await?;
renderer.end_frame().await?;
renderer.present().await?;
```

## Performance

### Batching Strategy
- Pre-allocated buffers: 64K vertices, 192K indices
- Automatic flush at 100 commands
- Single draw call for most frames

### Mobile Optimizations
- Minimal state changes
- Efficient buffer uploads
- Power-efficient frame batching
- Reduced memory allocations

## Future Enhancements

- [ ] Instanced rendering for particles
- [ ] Texture atlasing system
- [ ] Compute shader support (WebGL3)
- [ ] Multi-render target support
- [ ] Custom blend modes
- [ ] MSAA anti-aliasing

## Dependencies

- `playground-core-rendering`: Trait definitions
- `playground-core-types`: Shared types (Shared<T>)
- `nalgebra`: Matrix math operations
- `async-trait`: Async trait support

## License

Part of the Android Playground project.