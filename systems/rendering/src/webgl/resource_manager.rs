use std::collections::HashMap;
use web_sys::{WebGl2RenderingContext, WebGlBuffer, WebGlTexture, WebGlProgram, WebGlFramebuffer, WebGlVertexArrayObject};
use crate::resources::{Handle, TextureHandle, BufferHandle, ShaderHandle, PipelineHandle, RenderTargetHandle};
use crate::resources::{VertexBuffer, IndexBuffer, UniformBuffer, StorageBuffer};
use crate::error::RendererError;

pub struct WebGLResource<T> {
    pub handle: Handle<T>,
    pub generation: u32,
    pub gl_object: Option<T>,
}

pub struct ResourceManager {
    next_id: u64,
    
    // WebGL objects
    buffers: HashMap<u64, WebGlBuffer>,
    textures: HashMap<u64, WebGlTexture>,
    programs: HashMap<u64, WebGlProgram>,
    framebuffers: HashMap<u64, WebGlFramebuffer>,
    vaos: HashMap<u64, WebGlVertexArrayObject>,
    
    // Resource metadata
    vertex_buffers: HashMap<u64, VertexBuffer>,
    index_buffers: HashMap<u64, IndexBuffer>,
    uniform_buffers: HashMap<u64, UniformBuffer>,
    storage_buffers: HashMap<u64, StorageBuffer>,
    
    // Free lists for recycling
    free_buffer_ids: Vec<u64>,
    free_texture_ids: Vec<u64>,
    free_program_ids: Vec<u64>,
    free_framebuffer_ids: Vec<u64>,
}

impl ResourceManager {
    pub fn new() -> Self {
        Self {
            next_id: 1,
            buffers: HashMap::new(),
            textures: HashMap::new(),
            programs: HashMap::new(),
            framebuffers: HashMap::new(),
            vaos: HashMap::new(),
            vertex_buffers: HashMap::new(),
            index_buffers: HashMap::new(),
            uniform_buffers: HashMap::new(),
            storage_buffers: HashMap::new(),
            free_buffer_ids: Vec::new(),
            free_texture_ids: Vec::new(),
            free_program_ids: Vec::new(),
            free_framebuffer_ids: Vec::new(),
        }
    }

    pub fn allocate_buffer_id(&mut self) -> u64 {
        if let Some(id) = self.free_buffer_ids.pop() {
            id
        } else {
            let id = self.next_id;
            self.next_id += 1;
            id
        }
    }

    pub fn allocate_texture_id(&mut self) -> u64 {
        if let Some(id) = self.free_texture_ids.pop() {
            id
        } else {
            let id = self.next_id;
            self.next_id += 1;
            id
        }
    }

    pub fn allocate_program_id(&mut self) -> u64 {
        if let Some(id) = self.free_program_ids.pop() {
            id
        } else {
            let id = self.next_id;
            self.next_id += 1;
            id
        }
    }

    pub fn allocate_framebuffer_id(&mut self) -> u64 {
        if let Some(id) = self.free_framebuffer_ids.pop() {
            id
        } else {
            let id = self.next_id;
            self.next_id += 1;
            id
        }
    }

    pub fn store_buffer(&mut self, buffer: WebGlBuffer) -> BufferHandle {
        let id = self.allocate_buffer_id();
        self.buffers.insert(id, buffer);
        Handle::new(id, 0)
    }

    pub fn get_buffer(&self, handle: BufferHandle) -> Option<&WebGlBuffer> {
        self.buffers.get(&handle.id())
    }

    pub fn remove_buffer(&mut self, handle: BufferHandle) -> Option<WebGlBuffer> {
        let buffer = self.buffers.remove(&handle.id());
        if buffer.is_some() {
            self.free_buffer_ids.push(handle.id());
        }
        buffer
    }

    pub fn store_texture(&mut self, texture: WebGlTexture) -> TextureHandle {
        let id = self.allocate_texture_id();
        self.textures.insert(id, texture);
        Handle::new(id, 0)
    }

    pub fn get_texture(&self, handle: TextureHandle) -> Option<&WebGlTexture> {
        self.textures.get(&handle.id())
    }

    pub fn remove_texture(&mut self, handle: TextureHandle) -> Option<WebGlTexture> {
        let texture = self.textures.remove(&handle.id());
        if texture.is_some() {
            self.free_texture_ids.push(handle.id());
        }
        texture
    }

    pub fn store_program(&mut self, program: WebGlProgram) -> ShaderHandle {
        let id = self.allocate_program_id();
        self.programs.insert(id, program);
        Handle::new(id, 0)
    }

    pub fn get_program(&self, handle: ShaderHandle) -> Option<&WebGlProgram> {
        self.programs.get(&handle.id())
    }

    pub fn remove_program(&mut self, handle: ShaderHandle) -> Option<WebGlProgram> {
        let program = self.programs.remove(&handle.id());
        if program.is_some() {
            self.free_program_ids.push(handle.id());
        }
        program
    }

    pub fn store_framebuffer(&mut self, framebuffer: WebGlFramebuffer) -> RenderTargetHandle {
        let id = self.allocate_framebuffer_id();
        self.framebuffers.insert(id, framebuffer);
        Handle::new(id, 0)
    }

    pub fn get_framebuffer(&self, handle: RenderTargetHandle) -> Option<&WebGlFramebuffer> {
        self.framebuffers.get(&handle.id())
    }

    pub fn remove_framebuffer(&mut self, handle: RenderTargetHandle) -> Option<WebGlFramebuffer> {
        let framebuffer = self.framebuffers.remove(&handle.id());
        if framebuffer.is_some() {
            self.free_framebuffer_ids.push(handle.id());
        }
        framebuffer
    }

    pub fn store_vao(&mut self, vao: WebGlVertexArrayObject, pipeline_id: u64) {
        self.vaos.insert(pipeline_id, vao);
    }

    pub fn get_vao(&self, pipeline_id: u64) -> Option<&WebGlVertexArrayObject> {
        self.vaos.get(&pipeline_id)
    }

    pub fn store_vertex_buffer(&mut self, handle: BufferHandle, buffer: VertexBuffer) {
        self.vertex_buffers.insert(handle.id(), buffer);
    }

    pub fn get_vertex_buffer(&self, handle: &BufferHandle) -> Option<&VertexBuffer> {
        self.vertex_buffers.get(&handle.id())
    }

    pub fn store_index_buffer(&mut self, handle: BufferHandle, buffer: IndexBuffer) {
        self.index_buffers.insert(handle.id(), buffer);
    }

    pub fn get_index_buffer(&self, handle: &BufferHandle) -> Option<&IndexBuffer> {
        self.index_buffers.get(&handle.id())
    }

    pub fn store_uniform_buffer(&mut self, handle: BufferHandle, buffer: UniformBuffer) {
        self.uniform_buffers.insert(handle.id(), buffer);
    }

    pub fn get_uniform_buffer(&self, handle: &BufferHandle) -> Option<&UniformBuffer> {
        self.uniform_buffers.get(&handle.id())
    }

    pub fn clear(&mut self) {
        self.buffers.clear();
        self.textures.clear();
        self.programs.clear();
        self.framebuffers.clear();
        self.vaos.clear();
        self.vertex_buffers.clear();
        self.index_buffers.clear();
        self.uniform_buffers.clear();
        self.storage_buffers.clear();
    }
}