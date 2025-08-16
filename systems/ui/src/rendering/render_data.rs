//! Render data for UI elements

use nalgebra::{Vector2, Vector4};
use serde::{Deserialize, Serialize};

/// Render data that UI elements provide for batching
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RenderData {
    /// Vertex positions
    pub vertices: Vec<Vector2<f32>>,
    /// Vertex colors
    pub colors: Vec<Vector4<f32>>,
    /// Texture coordinates
    pub uvs: Vec<Vector2<f32>>,
    /// Indices for triangles
    pub indices: Vec<u32>,
    /// Texture ID if applicable
    pub texture_id: Option<u32>,
    /// Z-order for sorting
    pub z_order: f32,
    /// Scissor rect for clipping
    pub scissor_rect: Option<ScissorRect>,
}

impl RenderData {
    pub fn new() -> Self {
        Self {
            vertices: Vec::new(),
            colors: Vec::new(),
            uvs: Vec::new(),
            indices: Vec::new(),
            texture_id: None,
            z_order: 0.0,
            scissor_rect: None,
        }
    }
    
    /// Add a quad (common for UI elements)
    pub fn add_quad(
        &mut self,
        position: Vector2<f32>,
        size: Vector2<f32>,
        color: Vector4<f32>,
    ) {
        let base_index = self.vertices.len() as u32;
        
        // Add vertices (top-left, top-right, bottom-right, bottom-left)
        self.vertices.push(position);
        self.vertices.push(position + Vector2::new(size.x, 0.0));
        self.vertices.push(position + size);
        self.vertices.push(position + Vector2::new(0.0, size.y));
        
        // Add colors
        for _ in 0..4 {
            self.colors.push(color);
        }
        
        // Add default UVs
        self.uvs.push(Vector2::new(0.0, 0.0));
        self.uvs.push(Vector2::new(1.0, 0.0));
        self.uvs.push(Vector2::new(1.0, 1.0));
        self.uvs.push(Vector2::new(0.0, 1.0));
        
        // Add indices for two triangles
        self.indices.push(base_index);
        self.indices.push(base_index + 1);
        self.indices.push(base_index + 2);
        
        self.indices.push(base_index);
        self.indices.push(base_index + 2);
        self.indices.push(base_index + 3);
    }
    
    /// Add a rounded rectangle
    pub fn add_rounded_rect(
        &mut self,
        position: Vector2<f32>,
        size: Vector2<f32>,
        radius: f32,
        color: Vector4<f32>,
    ) {
        // For now, just draw a regular quad
        // TODO: Implement actual rounded corners with more vertices
        self.add_quad(position, size, color);
    }
    
    /// Add text (placeholder - actual text rendering needs SDF)
    pub fn add_text(
        &mut self,
        position: Vector2<f32>,
        text: &str,
        size: f32,
        color: Vector4<f32>,
    ) {
        // TODO: Implement actual text rendering with SDF
        // For now, just add a placeholder quad for each character
        let char_width = size * 0.6;
        let mut x = position.x;
        for _ in text.chars() {
            self.add_quad(
                Vector2::new(x, position.y),
                Vector2::new(char_width, size),
                color * 0.2, // Dimmed to show it's placeholder
            );
            x += char_width + 2.0;
        }
    }
    
    /// Add a line
    pub fn add_line(
        &mut self,
        start: Vector2<f32>,
        end: Vector2<f32>,
        thickness: f32,
        color: Vector4<f32>,
    ) {
        // Calculate perpendicular vector for thickness
        let dir = end - start;
        let perp = Vector2::new(-dir.y, dir.x).normalize() * (thickness * 0.5);
        
        let base_index = self.vertices.len() as u32;
        
        // Add vertices for line quad
        self.vertices.push(start - perp);
        self.vertices.push(start + perp);
        self.vertices.push(end + perp);
        self.vertices.push(end - perp);
        
        // Add colors
        for _ in 0..4 {
            self.colors.push(color);
        }
        
        // Add default UVs
        self.uvs.push(Vector2::new(0.0, 0.0));
        self.uvs.push(Vector2::new(0.0, 1.0));
        self.uvs.push(Vector2::new(1.0, 1.0));
        self.uvs.push(Vector2::new(1.0, 0.0));
        
        // Add indices
        self.indices.push(base_index);
        self.indices.push(base_index + 1);
        self.indices.push(base_index + 2);
        
        self.indices.push(base_index);
        self.indices.push(base_index + 2);
        self.indices.push(base_index + 3);
    }
    
    /// Merge another RenderData into this one
    pub fn merge(&mut self, other: &RenderData) {
        let vertex_offset = self.vertices.len() as u32;
        
        self.vertices.extend(&other.vertices);
        self.colors.extend(&other.colors);
        self.uvs.extend(&other.uvs);
        
        // Offset indices
        for index in &other.indices {
            self.indices.push(index + vertex_offset);
        }
    }
}

/// Scissor rectangle for clipping
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct ScissorRect {
    pub position: Vector2<f32>,
    pub size: Vector2<f32>,
}

impl ScissorRect {
    pub fn new(x: f32, y: f32, width: f32, height: f32) -> Self {
        Self {
            position: Vector2::new(x, y),
            size: Vector2::new(width, height),
        }
    }
}