
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Color {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

pub struct VertexBuffer {
    vertices: Vec<f32>,
    max_vertices: usize,
    vertex_size: usize,
}

impl VertexBuffer {
    pub fn new(max_vertices: usize) -> Self {
        Self {
            vertices: Vec::with_capacity(max_vertices * 9),
            max_vertices,
            vertex_size: 9,
        }
    }

    pub fn push_vertex(&mut self, x: f32, y: f32, u: f32, v: f32, color: Color) {
        self.vertices.push(x);
        self.vertices.push(y);
        self.vertices.push(u);
        self.vertices.push(v);
        self.vertices.push(color.r);
        self.vertices.push(color.g);
        self.vertices.push(color.b);
        self.vertices.push(color.a);
        self.vertices.push(0.0);
    }

    pub fn vertex_count(&self) -> usize {
        self.vertices.len() / self.vertex_size
    }

    pub fn clear(&mut self) {
        self.vertices.clear();
    }

    pub fn data(&self) -> &[f32] {
        &self.vertices
    }

    pub fn is_full(&self) -> bool {
        self.vertex_count() >= self.max_vertices
    }
}

pub struct IndexBuffer {
    indices: Vec<u16>,
    max_indices: usize,
}

impl IndexBuffer {
    pub fn new(max_indices: usize) -> Self {
        Self {
            indices: Vec::with_capacity(max_indices),
            max_indices,
        }
    }

    pub fn push_index(&mut self, index: u16) {
        self.indices.push(index);
    }

    pub fn push_triangle(&mut self, i0: u16, i1: u16, i2: u16) {
        self.indices.push(i0);
        self.indices.push(i1);
        self.indices.push(i2);
    }

    pub fn index_count(&self) -> usize {
        self.indices.len()
    }

    pub fn clear(&mut self) {
        self.indices.clear();
    }

    pub fn data(&self) -> &[u16] {
        &self.indices
    }

    pub fn is_full(&self) -> bool {
        self.index_count() >= self.max_indices
    }
}

pub struct UniformBuffer {
    data: Vec<u8>,
    size: usize,
}

impl UniformBuffer {
    pub fn new(size: usize) -> Self {
        Self {
            data: vec![0; size],
            size,
        }
    }

    pub fn update(&mut self, offset: usize, data: &[u8]) -> Result<(), String> {
        if offset + data.len() > self.size {
            return Err("UniformBuffer: data exceeds buffer size".into());
        }
        
        self.data[offset..offset + data.len()].copy_from_slice(data);
        Ok(())
    }

    pub fn data(&self) -> &[u8] {
        &self.data
    }

    pub fn size(&self) -> usize {
        self.size
    }
}