//! Wrapper types for avoiding dyn trait objects

use bytes::Bytes;
use serde::{Serialize, Deserialize};

/// Concrete wrapper for render targets that avoids dyn
/// This follows the NO dyn pattern used throughout the codebase
#[derive(Clone)]
pub struct RenderTargetWrapper {
    /// Serialized render target data
    data: Bytes,
    /// Unique handle/ID for this render target
    handle: u32,
    /// Width in pixels
    width: u32,
    /// Height in pixels
    height: u32,
    /// Format description (e.g., "RGBA8", "Depth24Stencil8")
    format: String,
    /// Target type (e.g., "Framebuffer", "Texture", "Screen")
    target_type: String,
}

impl RenderTargetWrapper {
    /// Create a new render target wrapper
    pub fn new(
        data: Bytes,
        handle: u32,
        width: u32,
        height: u32,
        format: String,
        target_type: String,
    ) -> Self {
        Self {
            data,
            handle,
            width,
            height,
            format,
            target_type,
        }
    }
    
    /// Get the render target handle
    pub fn handle(&self) -> u32 {
        self.handle
    }
    
    /// Get the width
    pub fn width(&self) -> u32 {
        self.width
    }
    
    /// Get the height
    pub fn height(&self) -> u32 {
        self.height
    }
    
    /// Get the format
    pub fn format(&self) -> &str {
        &self.format
    }
    
    /// Get the target type
    pub fn target_type(&self) -> &str {
        &self.target_type
    }
    
    /// Get the serialized data
    pub fn data(&self) -> &Bytes {
        &self.data
    }
    
    /// Update dimensions (for resize operations)
    pub fn resize(&mut self, width: u32, height: u32) {
        self.width = width;
        self.height = height;
    }
}

/// Metadata for render target operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RenderTargetInfo {
    pub handle: u32,
    pub width: u32,
    pub height: u32,
    pub format: String,
    pub target_type: String,
    pub is_default: bool,
    pub has_depth: bool,
    pub has_stencil: bool,
    pub sample_count: u32,
}

impl Default for RenderTargetInfo {
    fn default() -> Self {
        Self {
            handle: 0,
            width: 1920,
            height: 1080,
            format: "RGBA8".to_string(),
            target_type: "Screen".to_string(),
            is_default: true,
            has_depth: true,
            has_stencil: false,
            sample_count: 1,
        }
    }
}