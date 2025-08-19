# playground-systems-rendering

GPU rendering system with WebGL/Vulkan backends, render graphs, and single-draw-call batching.

## Overview

The Rendering System provides a high-performance, mobile-optimized graphics pipeline. It uses core/ecs internally for resource tracking and implements the BaseRenderer trait for multiple backends.

### Key Features
- BaseRenderer trait for backend abstraction
- WebGL 2.0 implementation (Vulkan planned)
- Render graph for pass organization
- Single draw call batching for mobile
- Texture streaming with LOD support
- Compute shader support (where available)
- Hot-reload for shaders
- ECS-based resource tracking
- Mobile-optimized with power efficiency

## Architecture

### BaseRenderer Trait
The core abstraction for all rendering backends:

```rust
use playground_systems_rendering::{BaseRenderer, RenderingSystem};

// RenderingSystem wraps any BaseRenderer implementation
let mut rendering = RenderingSystem::<WebGLRenderer>::new().await?;
rendering.set_renderer(WebGLRenderer::new()?);
rendering.initialize().await?;
```

### ECS Resource Management
All GPU resources are tracked as entities:

```rust
// Resources are entities with components
TextureResourceComponent → Tracks texture state
BufferResourceComponent → Tracks buffer allocations
ShaderResourceComponent → Tracks shader programs
PipelineResourceComponent → Tracks render pipelines
RenderTargetComponent → Tracks framebuffers
```

### Render Graph System
Organize rendering into passes:

```rust
// Define passes that execute in order
RenderGraph {
    passes: [
        ShadowPass,      // Render shadow maps
        GeometryPass,    // Render scene geometry
        LightingPass,    // Apply lighting
        PostProcessPass, // Effects
        UIPass,          // Overlay UI
    ]
}
```

## Usage

### Basic Setup
```rust
use playground_systems_rendering::{RenderingSystem, WebGLRenderer};
use playground_systems_rendering::graph::RenderGraph;

// Create rendering system with WebGL backend
let mut rendering = RenderingSystem::<WebGLRenderer>::new().await?;
let webgl = WebGLRenderer::new(canvas_element)?;
rendering.set_renderer(webgl);
rendering.initialize().await?;

// Create and set render graph
let graph = RenderGraph::new();
rendering.set_render_graph(graph)?;
```

### Creating Resources

#### Textures
```rust
use playground_systems_rendering::resources::{TextureDesc, TextureFormat};

// Create texture
let texture_desc = TextureDesc {
    width: 1024,
    height: 1024,
    format: TextureFormat::RGBA8,
    mip_levels: 10, // Full mip chain
    usage: TextureUsage::SAMPLED | TextureUsage::RENDER_TARGET,
};

let texture = rendering.create_texture(&texture_desc)?;

// Update texture data
let pixel_data = vec![255u8; 1024 * 1024 * 4]; // White texture
rendering.update_texture(texture, &pixel_data, None)?;

// Stream specific mip level
rendering.request_texture_lod(texture, 5).await?;
```

#### Buffers
```rust
use playground_systems_rendering::resources::buffer::{VertexFormat, IndexType};

// Define vertex format
let vertex_format = VertexFormat::new()
    .attribute("position", AttributeType::Float3)
    .attribute("normal", AttributeType::Float3)
    .attribute("uv", AttributeType::Float2);

// Create vertex buffer
let vertices = generate_mesh_vertices();
let vertex_buffer = rendering.create_vertex_buffer(&vertices, &vertex_format)?;

// Create index buffer
let indices: Vec<u16> = generate_mesh_indices();
let index_buffer = rendering.create_index_buffer(
    bytemuck::cast_slice(&indices),
    IndexType::U16
)?;

// Create uniform buffer for matrices
let uniform_buffer = rendering.create_uniform_buffer(
    std::mem::size_of::<[f32; 16]>() * 2 // View + Projection
)?;
```

#### Shaders
```rust
use playground_systems_rendering::resources::{ShaderStage};

// Load vertex and fragment shaders
let vertex_shader = rendering.create_shader(
    "shaders/default.vert",
    ShaderStage::Vertex
)?;

let fragment_shader = rendering.create_shader(
    "shaders/default.frag",
    ShaderStage::Fragment
)?;

// Create compute shader (if supported)
let compute_shader = rendering.create_compute_shader(
    "shaders/lighting.comp"
)?;

// Enable hot-reload
rendering.register_shader_watch(vertex_shader)?;
```

#### Pipelines
```rust
use playground_systems_rendering::resources::{PipelineDesc};
use playground_systems_rendering::state::{BlendState, DepthStencilState};

// Create render pipeline
let pipeline_desc = PipelineDesc {
    vertex_shader,
    fragment_shader,
    vertex_format: vertex_format.clone(),
    primitive_topology: PrimitiveTopology::Triangles,
    blend_state: BlendState::alpha_blend(),
    depth_stencil: DepthStencilState::default(),
    cull_mode: CullMode::Back,
};

let pipeline = rendering.create_pipeline(&pipeline_desc)?;
```

### Render Passes

#### Creating Custom Pass
```rust
use playground_systems_rendering::graph::pass::{Pass, RenderPass};
use playground_systems_rendering::commands::CommandBuffer;

pub struct GeometryPass {
    pipeline: PipelineHandle,
    render_target: RenderTargetHandle,
}

#[async_trait]
impl Pass for GeometryPass {
    async fn execute(&mut self, cmd: &mut dyn CommandBuffer) -> Result<()> {
        // Set render target
        cmd.set_render_target(self.render_target);
        
        // Clear
        cmd.clear(ClearFlags::COLOR | ClearFlags::DEPTH, [0.1, 0.1, 0.2, 1.0]);
        
        // Bind pipeline
        cmd.set_pipeline(self.pipeline);
        
        // Set uniforms
        cmd.set_uniform_buffer(0, view_proj_buffer);
        
        // Draw all geometry (batched)
        cmd.draw_indexed(index_buffer, vertex_buffer, index_count);
        
        Ok(())
    }
    
    fn dependencies(&self) -> Vec<PassId> {
        vec![] // No dependencies
    }
}

// Add to render graph
let pass = Box::new(GeometryPass { pipeline, render_target });
let pass_id = rendering.add_render_pass(pass)?;
```

#### Shadow Mapping
```rust
// Create shadow map texture
let shadow_map = rendering.create_texture(&TextureDesc {
    width: 2048,
    height: 2048,
    format: TextureFormat::Depth32F,
    usage: TextureUsage::RENDER_TARGET | TextureUsage::SAMPLED,
    ..Default::default()
})?;

// Shadow pass renders to depth texture
pub struct ShadowPass {
    shadow_map: TextureHandle,
    light_matrices: UniformBuffer,
}

impl Pass for ShadowPass {
    async fn execute(&mut self, cmd: &mut dyn CommandBuffer) -> Result<()> {
        cmd.set_render_target_texture(self.shadow_map);
        cmd.clear(ClearFlags::DEPTH, [0.0; 4]);
        cmd.set_pipeline(self.shadow_pipeline);
        cmd.draw_indexed(/* shadow casters */);
        Ok(())
    }
}
```

### Single Draw Call Batching

The system optimizes for mobile by batching everything into one draw call:

```rust
// All meshes are packed into a single buffer
let mega_buffer = rendering.create_vertex_buffer(
    &all_vertices_concatenated,
    &vertex_format
)?;

// Instance data for each object
let instance_buffer = rendering.create_storage_buffer(
    instance_data.len() * std::mem::size_of::<InstanceData>(),
    true // read-only
)?;

// Single draw call renders everything
cmd.draw_indexed_instanced(
    mega_index_buffer,
    mega_vertex_buffer,
    instance_buffer,
    total_indices,
    instance_count
);
```

### Compute Shaders

For GPUs that support compute:

```rust
use playground_systems_rendering::compute::ComputeResources;

// Create compute resources
let mut resources = ComputeResources::new();
resources.add_storage_buffer(0, particle_buffer);
resources.add_texture(1, velocity_texture);

// Dispatch compute work
rendering.dispatch_compute(
    64, 64, 1, // Work groups
    compute_shader,
    &resources
)?;
```

### Texture Streaming

Efficient texture loading with LOD:

```rust
// Set streaming budget (in bytes)
rendering.set_streaming_budget(100 * 1024 * 1024); // 100MB

// Update priorities based on distance
let mut priorities = HashMap::new();
priorities.insert(texture_near, 1.0);  // High priority
priorities.insert(texture_far, 0.1);   // Low priority
rendering.update_streaming_priorities(priorities);

// Request specific LOD level
let promise = rendering.request_texture_lod(texture, 3);
promise.await?; // Wait for LOD to load
```

### Command Buffers

Record and submit rendering commands:

```rust
// Create command buffer
let cmd_buffer = rendering.create_command_buffer();

// Record commands
{
    let mut cmd = cmd_buffer.write().await;
    cmd.set_viewport(0, 0, 1920, 1080);
    cmd.set_pipeline(pipeline);
    cmd.bind_texture(0, texture);
    cmd.draw(vertex_count);
}

// Submit with dependencies
rendering.submit_command_buffer(cmd_buffer, vec![previous_sync_point]);
```

### Frame Lifecycle

```rust
// Main render loop
loop {
    // Begin frame
    rendering.begin_frame()?;
    
    // Execute render graph
    rendering.render(&render_graph)?;
    
    // End frame
    rendering.end_frame()?;
    
    // Present to screen
    rendering.present()?;
    
    // Get metrics
    let metrics = rendering.get_metrics();
    println!("Draw calls: {}, Triangles: {}", 
        metrics.draw_calls, metrics.triangles);
}
```

## Components Reference

### TextureResourceComponent
```rust
pub struct TextureResourceComponent {
    pub handle: TextureHandle,
    pub width: u32,
    pub height: u32,
    pub format: TextureFormat,
    pub mip_levels: u32,
    pub memory_size: usize,
    pub last_used_frame: u64,
    pub streaming_priority: f32,
}
```

### BufferResourceComponent
```rust
pub struct BufferResourceComponent {
    pub buffer_type: BufferType,
    pub size: usize,
    pub usage: BufferUsage,
    pub last_updated_frame: u64,
    pub cpu_visible: bool,
}
```

### FrameStateComponent
```rust
pub struct FrameStateComponent {
    pub frame_number: u64,
    pub frame_time_ms: f32,
    pub cpu_time_ms: f32,
    pub gpu_time_ms: f32,
    pub draw_calls: u32,
    pub triangles_rendered: u64,
    pub state_changes: u32,
    pub texture_memory_used: usize,
    pub buffer_memory_used: usize,
}
```

## WebGL Implementation

The WebGL backend provides browser rendering:

```rust
use playground_systems_rendering::webgl::{WebGLRenderer, WebGLContext};

// Create WebGL2 context
let canvas = document.get_element_by_id("canvas");
let context = WebGLContext::new(canvas, WebGLContextOptions {
    alpha: false,
    antialias: false, // We do our own AA
    depth: true,
    stencil: true,
    power_preference: "high-performance",
})?;

// Create renderer
let webgl = WebGLRenderer::with_context(context);
```

### WebGL Limitations
- No compute shaders (use fragment shader workarounds)
- Limited uniform buffer size (16KB typical)
- No geometry shaders
- Texture format restrictions
- No async texture uploads

## Performance Optimizations

### Mobile-First Design
- **Single Draw Call**: Entire frame in one call
- **Texture Atlasing**: Reduce texture switches
- **Instanced Rendering**: For repeated geometry
- **LOD System**: Lower detail at distance
- **Occlusion Culling**: Don't render hidden objects
- **Frame Batching**: Group state changes

### Memory Management
```rust
// Set memory budgets
rendering.set_texture_budget(200 * 1024 * 1024);  // 200MB
rendering.set_buffer_budget(50 * 1024 * 1024);    // 50MB

// Manual resource cleanup
rendering.destroy_texture(old_texture)?;
rendering.destroy_buffer(old_buffer)?;
```

### Profiling
```rust
// Enable GPU timing
rendering.enable_gpu_timing(true);

// Get detailed metrics
let metrics = rendering.get_metrics();
println!("GPU Time: {}ms", metrics.gpu_time_ms);
println!("State Changes: {}", metrics.state_changes);
println!("Memory Used: {}MB", metrics.total_memory_mb());
```

## Testing

```rust
#[tokio::test]
async fn test_rendering_system() {
    // Create mock renderer
    let mock = MockRenderer::new();
    
    // Initialize system
    let mut rendering = RenderingSystem::new().await.unwrap();
    rendering.set_renderer(mock);
    rendering.initialize().await.unwrap();
    
    // Create resources
    let texture = rendering.create_texture(&TextureDesc::default()).unwrap();
    
    // Verify ECS tracking
    let entities = rendering.query_texture_entities().await.unwrap();
    assert_eq!(entities.len(), 1);
}
```

## Future: Vulkan Backend

```rust
// Planned Vulkan implementation
let vulkan = VulkanRenderer::new(VulkanConfig {
    device_extensions: vec!["VK_KHR_swapchain"],
    validation_layers: cfg!(debug_assertions),
    present_mode: PresentMode::Mailbox, // Triple buffering
})?;

// Better mobile GPU features
- Tile-based rendering
- Async compute
- Mesh shaders
- Variable rate shading
```

## Architecture Rules

- Uses core/ecs for resource tracking
- BaseRenderer trait for backend abstraction
- Thread-safe with Arc<RwLock<>>
- All operations return Result<T, RendererError>
- Single draw call target for mobile
- NO unsafe code (except FFI to GPU drivers)
- Async operations where beneficial

## Dependencies

- `playground-core-ecs`: Resource entity tracking
- `playground-core-types`: Shared types
- `nalgebra`: Matrix/vector math
- `bytemuck`: Safe transmutation for GPU data
- `web-sys`: WebGL bindings (with webgl feature)
- `tokio`: Async runtime
- `async-trait`: Async trait implementations

## See Also

- [systems/ui](../ui/README.md) - UI rendering integration
- [plugins/rendering-demos](../../plugins/rendering-demos/README.md) - Example usage
- [WebGL Spec](https://www.khronos.org/webgl/) - WebGL documentation