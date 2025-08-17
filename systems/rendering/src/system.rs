//! Rendering system with ECS integration for internal state management
//!
//! This module provides the RenderingSystem which uses core/ecs World for
//! tracking internal state while maintaining full compatibility with the
//! existing BaseRenderer trait and implementations.

use std::sync::Arc;
use std::collections::HashMap;
use playground_ecs::{World, EntityId, ComponentRegistry};

use crate::base_renderer::BaseRenderer;
use crate::components::*;
use crate::error::RendererError;
use crate::resources::{TextureHandle, ShaderHandle, PipelineHandle, RenderTargetHandle};
use crate::resources::buffer::{VertexBuffer, IndexBuffer, UniformBuffer, StorageBuffer};
use crate::graph::pass::PassId;
use crate::capabilities::RendererCapabilities;

/// The main rendering system that manages GPU resources and render state using ECS
pub struct RenderingSystem<R: BaseRenderer<Error = RendererError>> {
    /// ECS World for internal state management
    world: Arc<World>,
    
    /// The actual renderer implementation (WebGL, Vulkan, etc)
    renderer: Option<R>,
    
    /// Mapping from resource handles to ECS entities
    texture_entities: HashMap<TextureHandle, EntityId>,
    buffer_entities: HashMap<u64, EntityId>, // Buffer ID to entity
    shader_entities: HashMap<ShaderHandle, EntityId>,
    pipeline_entities: HashMap<PipelineHandle, EntityId>,
    render_target_entities: HashMap<RenderTargetHandle, EntityId>,
    pass_entities: HashMap<PassId, EntityId>,
    
    /// Entity tracking frame state
    frame_entity: Option<EntityId>,
    
    /// Entity tracking renderer capabilities
    capabilities_entity: Option<EntityId>,
    
    /// Initialized flag
    initialized: bool,
}

impl<R: BaseRenderer<Error = RendererError>> RenderingSystem<R> {
    /// Create a new rendering system with ECS integration
    pub async fn new() -> Result<Self, RendererError> {
        let registry = Arc::new(ComponentRegistry::new());
        let world = Arc::new(World::with_registry(registry));
        
        // Register all rendering components
        world.register_component::<TextureResourceComponent>().await
            .map_err(|e| RendererError::InitializationFailed(e.to_string()))?;
        world.register_component::<BufferResourceComponent>().await
            .map_err(|e| RendererError::InitializationFailed(e.to_string()))?;
        world.register_component::<ShaderResourceComponent>().await
            .map_err(|e| RendererError::InitializationFailed(e.to_string()))?;
        world.register_component::<PipelineResourceComponent>().await
            .map_err(|e| RendererError::InitializationFailed(e.to_string()))?;
        world.register_component::<RenderTargetComponent>().await
            .map_err(|e| RendererError::InitializationFailed(e.to_string()))?;
        world.register_component::<RenderPassComponent>().await
            .map_err(|e| RendererError::InitializationFailed(e.to_string()))?;
        world.register_component::<FrameStateComponent>().await
            .map_err(|e| RendererError::InitializationFailed(e.to_string()))?;
        world.register_component::<CapabilitiesComponent>().await
            .map_err(|e| RendererError::InitializationFailed(e.to_string()))?;
        world.register_component::<StreamingPriorityComponent>().await
            .map_err(|e| RendererError::InitializationFailed(e.to_string()))?;
        
        Ok(Self {
            world,
            renderer: None,
            texture_entities: HashMap::new(),
            buffer_entities: HashMap::new(),
            shader_entities: HashMap::new(),
            pipeline_entities: HashMap::new(),
            render_target_entities: HashMap::new(),
            pass_entities: HashMap::new(),
            frame_entity: None,
            capabilities_entity: None,
            initialized: false,
        })
    }
    
    /// Set the renderer backend (WebGL, Vulkan, etc)
    pub fn set_renderer(&mut self, renderer: R) {
        self.renderer = Some(renderer);
    }
    
    /// Get a reference to the renderer
    pub fn renderer(&self) -> Option<&R> {
        self.renderer.as_ref()
    }
    
    /// Get a mutable reference to the renderer
    pub fn renderer_mut(&mut self) -> Option<&mut R> {
        self.renderer.as_mut()
    }
    
    /// Initialize the rendering system and backend
    pub async fn initialize(&mut self) -> Result<(), RendererError> {
        if self.initialized {
            return Ok(());
        }
        
        // Initialize the renderer backend
        if let Some(renderer) = &mut self.renderer {
            renderer.initialize()?;
            
            // Create frame state entity
            let frame_component = FrameStateComponent {
                frame_number: 0,
                frame_time_ms: 0.0,
                cpu_time_ms: 0.0,
                gpu_time_ms: 0.0,
                draw_calls: 0,
                triangles_rendered: 0,
                state_changes: 0,
                texture_memory_used: 0,
                buffer_memory_used: 0,
            };
            
            let entities = self.world.spawn_batch(vec![
                vec![Box::new(frame_component) as playground_ecs::ComponentBox],
            ]).await.map_err(|e| RendererError::InitializationFailed(e.to_string()))?;
            
            self.frame_entity = entities.first().copied();
            
            // Create capabilities entity
            let caps = renderer.capabilities();
            let caps_component = CapabilitiesComponent {
                max_texture_size: caps.max_texture_size,
                max_vertex_attributes: caps.max_vertex_attributes,
                max_uniform_buffer_size: caps.max_uniform_buffer_size as u32,
                max_storage_buffer_size: caps.max_storage_buffer_size as u32,
                features: caps.features.clone(),
                renderer_name: "WebGL2".to_string(), // Would be queried from actual renderer
                vendor: "Unknown".to_string(),
            };
            
            let entities = self.world.spawn_batch(vec![
                vec![Box::new(caps_component) as playground_ecs::ComponentBox],
            ]).await.map_err(|e| RendererError::InitializationFailed(e.to_string()))?;
            
            self.capabilities_entity = entities.first().copied();
        } else {
            return Err(RendererError::InitializationFailed("No renderer backend set".to_string()));
        }
        
        self.initialized = true;
        Ok(())
    }
    
    /// Shutdown the rendering system
    pub async fn shutdown(&mut self) -> Result<(), RendererError> {
        if let Some(renderer) = &mut self.renderer {
            renderer.shutdown()?;
        }
        
        // Clean up ECS entities
        let mut entities_to_remove = Vec::new();
        entities_to_remove.extend(self.texture_entities.values().copied());
        entities_to_remove.extend(self.buffer_entities.values().copied());
        entities_to_remove.extend(self.shader_entities.values().copied());
        entities_to_remove.extend(self.pipeline_entities.values().copied());
        entities_to_remove.extend(self.render_target_entities.values().copied());
        entities_to_remove.extend(self.pass_entities.values().copied());
        
        if let Some(entity) = self.frame_entity {
            entities_to_remove.push(entity);
        }
        if let Some(entity) = self.capabilities_entity {
            entities_to_remove.push(entity);
        }
        
        self.world.despawn_batch(entities_to_remove).await
            .map_err(|e| RendererError::ResourceCreationFailed(format!("Failed to despawn entities: {}", e)))?;
        
        self.texture_entities.clear();
        self.buffer_entities.clear();
        self.shader_entities.clear();
        self.pipeline_entities.clear();
        self.render_target_entities.clear();
        self.pass_entities.clear();
        self.frame_entity = None;
        self.capabilities_entity = None;
        self.initialized = false;
        
        Ok(())
    }
    
    /// Create a texture and track it in ECS
    pub async fn create_texture(
        &mut self,
        desc: &crate::resources::TextureDesc
    ) -> Result<TextureHandle, RendererError> {
        let renderer = self.renderer.as_mut()
            .ok_or_else(|| RendererError::InitializationFailed("No renderer backend".to_string()))?;
        
        // Create the texture in the backend
        let handle = renderer.create_texture(desc)?;
        
        // Create ECS component to track the texture
        let component = TextureResourceComponent {
            handle,
            width: desc.width,
            height: desc.height,
            format: 0, // Would map from desc.format
            mip_levels: desc.mip_levels,
            current_lod: 0,
            memory_usage: (desc.width * desc.height * 4) as usize, // Rough estimate
            last_used_frame: 0,
            debug_name: None,
        };
        
        let entities = self.world.spawn_batch(vec![
            vec![Box::new(component) as playground_ecs::ComponentBox],
        ]).await.map_err(|e| RendererError::ResourceCreationFailed(e.to_string()))?;
        
        if let Some(entity) = entities.first() {
            self.texture_entities.insert(handle, *entity);
        }
        
        Ok(handle)
    }
    
    /// Create a vertex buffer and track it in ECS
    pub async fn create_vertex_buffer(
        &mut self,
        data: &[u8],
        format: &crate::resources::buffer::VertexFormat
    ) -> Result<VertexBuffer, RendererError> {
        let renderer = self.renderer.as_mut()
            .ok_or_else(|| RendererError::InitializationFailed("No renderer backend".to_string()))?;
        
        // Create the buffer in the backend
        let buffer = renderer.create_vertex_buffer(data, format)?;
        
        // Create ECS component to track the buffer
        let component = BufferResourceComponent {
            buffer_type: BufferType::Vertex(buffer.clone()),
            size: data.len(),
            usage_flags: 0, // Would be set based on usage hints
            memory_usage: data.len(),
            last_updated_frame: 0,
            debug_name: None,
        };
        
        let entities = self.world.spawn_batch(vec![
            vec![Box::new(component) as playground_ecs::ComponentBox],
        ]).await.map_err(|e| RendererError::ResourceCreationFailed(e.to_string()))?;
        
        if let Some(entity) = entities.first() {
            self.buffer_entities.insert(buffer.handle.id(), *entity);
        }
        
        Ok(buffer)
    }
    
    /// Create a shader and track it in ECS
    pub async fn create_shader(
        &mut self,
        path: &str,
        stage: crate::resources::ShaderStage
    ) -> Result<ShaderHandle, RendererError> {
        let renderer = self.renderer.as_mut()
            .ok_or_else(|| RendererError::InitializationFailed("No renderer backend".to_string()))?;
        
        // Create the shader in the backend
        let handle = renderer.create_shader(path, stage)?;
        
        // Create ECS component to track the shader
        let component = ShaderResourceComponent {
            handle,
            path: path.into(),
            stage: stage as u32,
            compiled: true,
            last_modified: None,
            watch_enabled: false,
            compilation_errors: Vec::new(),
            debug_name: None,
        };
        
        let entities = self.world.spawn_batch(vec![
            vec![Box::new(component) as playground_ecs::ComponentBox],
        ]).await.map_err(|e| RendererError::ResourceCreationFailed(e.to_string()))?;
        
        if let Some(entity) = entities.first() {
            self.shader_entities.insert(handle, *entity);
        }
        
        Ok(handle)
    }
    
    /// Update frame metrics in ECS
    pub async fn update_frame_metrics(&mut self) -> Result<(), RendererError> {
        if let Some(renderer) = &self.renderer {
            let metrics = renderer.get_metrics();
            
            if let Some(entity) = self.frame_entity {
                // Get the current frame component
                let component_id = std::any::TypeId::of::<FrameStateComponent>();
                let _component_box = self.world.get_component_raw(entity, component_id).await
                    .map_err(|e| RendererError::ResourceCreationFailed(format!("Failed to get component: {}", e)))?;
                
                // Update with new metrics
                let new_component = FrameStateComponent {
                    frame_number: 0, // Would track this separately
                    frame_time_ms: metrics.frame_time_ms,
                    cpu_time_ms: 0.0, // Would measure separately
                    gpu_time_ms: 0.0, // Would measure separately
                    draw_calls: metrics.draw_calls,
                    triangles_rendered: metrics.triangles_rendered as u64,
                    state_changes: metrics.state_changes,
                    texture_memory_used: metrics.texture_uploads_bytes,
                    buffer_memory_used: metrics.buffer_uploads_bytes,
                };
                
                // Replace the component
                self.world.remove_component_raw(entity, component_id).await
                    .map_err(|e| RendererError::ResourceCreationFailed(format!("Failed to remove component: {}", e)))?;
                self.world.add_component_raw(
                    entity,
                    Box::new(new_component) as playground_ecs::ComponentBox,
                    component_id
                ).await.map_err(|e| RendererError::ResourceCreationFailed(format!("Failed to add component: {}", e)))?
            }
        }
        
        Ok(())
    }
    
    /// Query textures by their properties using ECS
    pub async fn query_textures_by_size(&self, min_size: u32) -> Vec<TextureHandle> {
        // This would use the ECS query system to find textures larger than min_size
        // For now, returning empty as this is just showing the pattern
        Vec::new()
    }
    
    /// Get total memory usage from all tracked resources
    pub async fn get_total_memory_usage(&self) -> Result<usize, RendererError> {
        if let Some(entity) = self.frame_entity {
            let component_id = std::any::TypeId::of::<FrameStateComponent>();
            let _component_box = self.world.get_component_raw(entity, component_id).await
                .map_err(|e| RendererError::ResourceCreationFailed(format!("Failed to get frame component: {}", e)))?;
            
            // Would downcast and extract memory usage
            // For now just return a placeholder
            Ok(0)
        } else {
            Ok(0)
        }
    }
    
    /// Run garbage collection on unused resources
    pub async fn collect_unused_resources(&mut self) -> Result<usize, RendererError> {
        let collected = self.world.run_gc().await
            .map_err(|e| RendererError::ResourceCreationFailed(format!("Failed to run GC: {}", e)))?;
        Ok(collected)
    }
    
    /// Get the ECS world for advanced queries
    pub fn world(&self) -> &Arc<World> {
        &self.world
    }
}