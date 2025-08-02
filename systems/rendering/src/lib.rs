//! Rendering System
//! 
//! This crate provides graphics rendering functionality for the playground system.

use playground_types::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum RenderingError {
    #[error("Rendering initialization failed: {0}")]
    InitializationFailed(String),
    #[error("Shader compilation failed: {0}")]
    ShaderCompilationFailed(String),
    #[error("Texture loading failed: {0}")]
    TextureLoadingFailed(String),
    #[error("Invalid render target: {0}")]
    InvalidRenderTarget(String),
    #[error("Drawing error: {0}")]
    DrawingError(String),
}

pub type RenderingResult<T> = Result<T, RenderingError>;

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Color {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

impl Color {
    pub fn new(r: f32, g: f32, b: f32, a: f32) -> Self {
        Self { r, g, b, a }
    }

    pub fn rgb(r: f32, g: f32, b: f32) -> Self {
        Self { r, g, b, a: 1.0 }
    }

    pub const WHITE: Color = Color { r: 1.0, g: 1.0, b: 1.0, a: 1.0 };
    pub const BLACK: Color = Color { r: 0.0, g: 0.0, b: 0.0, a: 1.0 };
    pub const RED: Color = Color { r: 1.0, g: 0.0, b: 0.0, a: 1.0 };
    pub const GREEN: Color = Color { r: 0.0, g: 1.0, b: 0.0, a: 1.0 };
    pub const BLUE: Color = Color { r: 0.0, g: 0.0, b: 1.0, a: 1.0 };
    pub const TRANSPARENT: Color = Color { r: 0.0, g: 0.0, b: 0.0, a: 0.0 };
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transform {
    pub position: (f32, f32, f32),
    pub rotation: (f32, f32, f32),
    pub scale: (f32, f32, f32),
}

impl Transform {
    pub fn new() -> Self {
        Self {
            position: (0.0, 0.0, 0.0),
            rotation: (0.0, 0.0, 0.0),
            scale: (1.0, 1.0, 1.0),
        }
    }

    pub fn with_position(mut self, x: f32, y: f32, z: f32) -> Self {
        self.position = (x, y, z);
        self
    }

    pub fn with_rotation(mut self, x: f32, y: f32, z: f32) -> Self {
        self.rotation = (x, y, z);
        self
    }

    pub fn with_scale(mut self, x: f32, y: f32, z: f32) -> Self {
        self.scale = (x, y, z);
        self
    }
}

impl Default for Transform {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone)]
pub struct RenderObject {
    pub id: String,
    pub transform: Transform,
    pub color: Color,
    pub visible: bool,
    pub layer: i32,
}

impl RenderObject {
    pub fn new(id: String) -> Self {
        Self {
            id,
            transform: Transform::new(),
            color: Color::WHITE,
            visible: true,
            layer: 0,
        }
    }

    pub fn with_transform(mut self, transform: Transform) -> Self {
        self.transform = transform;
        self
    }

    pub fn with_color(mut self, color: Color) -> Self {
        self.color = color;
        self
    }

    pub fn with_layer(mut self, layer: i32) -> Self {
        self.layer = layer;
        self
    }
}

#[derive(Debug, Clone)]
pub struct Camera {
    pub position: (f32, f32, f32),
    pub target: (f32, f32, f32),
    pub up: (f32, f32, f32),
    pub fov: f32,
    pub near_plane: f32,
    pub far_plane: f32,
}

impl Camera {
    pub fn new() -> Self {
        Self {
            position: (0.0, 0.0, 5.0),
            target: (0.0, 0.0, 0.0),
            up: (0.0, 1.0, 0.0),
            fov: 45.0,
            near_plane: 0.1,
            far_plane: 1000.0,
        }
    }
}

impl Default for Camera {
    fn default() -> Self {
        Self::new()
    }
}

/// Main rendering system struct
pub struct RenderingSystem {
    render_objects: HashMap<String, RenderObject>,
    camera: Camera,
    clear_color: Color,
    viewport_size: (u32, u32),
    initialized: bool,
}

impl RenderingSystem {
    /// Create a new rendering system
    pub fn new() -> Self {
        Self {
            render_objects: HashMap::new(),
            camera: Camera::new(),
            clear_color: Color::BLACK,
            viewport_size: (800, 600),
            initialized: false,
        }
    }

    /// Initialize the rendering system
    pub fn initialize(&mut self, viewport_width: u32, viewport_height: u32) -> RenderingResult<()> {
        if self.initialized {
            return Err(RenderingError::InitializationFailed("Already initialized".to_string()));
        }
        
        self.viewport_size = (viewport_width, viewport_height);
        self.initialized = true;
        Ok(())
    }

    /// Add a render object
    pub fn add_render_object(&mut self, object: RenderObject) -> RenderingResult<()> {
        if self.render_objects.contains_key(&object.id) {
            return Err(RenderingError::InvalidRenderTarget(
                format!("Render object with id '{}' already exists", object.id)
            ));
        }
        
        self.render_objects.insert(object.id.clone(), object);
        Ok(())
    }

    /// Remove a render object
    pub fn remove_render_object(&mut self, object_id: &str) -> RenderingResult<()> {
        if self.render_objects.remove(object_id).is_none() {
            return Err(RenderingError::InvalidRenderTarget(
                format!("Render object with id '{}' not found", object_id)
            ));
        }
        Ok(())
    }

    /// Get a render object
    pub fn get_render_object(&self, object_id: &str) -> Option<&RenderObject> {
        self.render_objects.get(object_id)
    }

    /// Get a mutable reference to a render object
    pub fn get_render_object_mut(&mut self, object_id: &str) -> Option<&mut RenderObject> {
        self.render_objects.get_mut(object_id)
    }

    /// Set the camera
    pub fn set_camera(&mut self, camera: Camera) {
        self.camera = camera;
    }

    /// Get the current camera
    pub fn get_camera(&self) -> &Camera {
        &self.camera
    }

    /// Get a mutable reference to the camera
    pub fn get_camera_mut(&mut self) -> &mut Camera {
        &mut self.camera
    }

    /// Set the clear color
    pub fn set_clear_color(&mut self, color: Color) {
        self.clear_color = color;
    }

    /// Set the viewport size
    pub fn set_viewport_size(&mut self, width: u32, height: u32) {
        self.viewport_size = (width, height);
    }

    /// Get the viewport size
    pub fn get_viewport_size(&self) -> (u32, u32) {
        self.viewport_size
    }

    /// Render a frame
    pub fn render(&mut self) -> RenderingResult<()> {
        if !self.initialized {
            return Err(RenderingError::InitializationFailed("Rendering system not initialized".to_string()));
        }

        // Clear the screen
        self.clear()?;

        // Sort render objects by layer
        let mut objects: Vec<&RenderObject> = self.render_objects.values()
            .filter(|obj| obj.visible)
            .collect();
        objects.sort_by_key(|obj| obj.layer);

        // Render each object
        for object in objects {
            self.render_object(object)?;
        }

        // Present the frame
        self.present()?;

        Ok(())
    }

    /// Clear the screen
    fn clear(&self) -> RenderingResult<()> {
        // TODO: Implement actual screen clearing
        Ok(())
    }

    /// Render a single object
    fn render_object(&self, _object: &RenderObject) -> RenderingResult<()> {
        // TODO: Implement actual object rendering
        Ok(())
    }

    /// Present the rendered frame
    fn present(&self) -> RenderingResult<()> {
        // TODO: Implement actual frame presentation
        Ok(())
    }

    /// Update the rendering system
    pub fn update(&mut self, _delta_time: f32) -> RenderingResult<()> {
        if !self.initialized {
            return Err(RenderingError::InitializationFailed("Rendering system not initialized".to_string()));
        }

        // TODO: Implement rendering system updates (animations, etc.)
        Ok(())
    }

    /// Get all render objects
    pub fn get_all_render_objects(&self) -> Vec<&RenderObject> {
        self.render_objects.values().collect()
    }

    /// Set object visibility
    pub fn set_object_visibility(&mut self, object_id: &str, visible: bool) -> RenderingResult<()> {
        if let Some(object) = self.render_objects.get_mut(object_id) {
            object.visible = visible;
            Ok(())
        } else {
            Err(RenderingError::InvalidRenderTarget(
                format!("Render object with id '{}' not found", object_id)
            ))
        }
    }
}

impl Default for RenderingSystem {
    fn default() -> Self {
        Self::new()
    }
}