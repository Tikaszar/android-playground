//! Type aliases used throughout the rendering system

use serde::{Serialize, Deserialize};

// Numeric type aliases
pub type Float = f32;
pub type Double = f64;
pub type Int = i32;
pub type UInt = u32;
pub type Index = u32;
pub type Byte = u8;
pub type ResourceId = u32;

// Vector/Matrix type aliases
pub type Vec2 = [Float; 2];
pub type Vec3 = [Float; 3];
pub type Vec4 = [Float; 4];
pub type Quat = [Float; 4];  // Quaternion [x, y, z, w]
pub type Mat2 = [[Float; 2]; 2];
pub type Mat3 = [[Float; 3]; 3];
pub type Mat4 = [[Float; 4]; 4];

// Color type aliases
pub type ColorRGB = [Float; 3];
pub type ColorRGBA = [Float; 4];
pub type ColorRGB8 = [Byte; 3];
pub type ColorRGBA8 = [Byte; 4];

// Common data structures (NOT components, just data)

/// Viewport for rendering
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Viewport {
    pub x: UInt,
    pub y: UInt,
    pub width: UInt,
    pub height: UInt,
}

/// Rectangle (for UI, sprites, etc.)
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Rect {
    pub x: Float,
    pub y: Float,
    pub width: Float,
    pub height: Float,
}

/// Bounding box for 3D
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct BoundingBox {
    pub min: Vec3,
    pub max: Vec3,
}

/// Bounding sphere for 3D
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct BoundingSphere {
    pub center: Vec3,
    pub radius: Float,
}

/// Simple color struct for convenience
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Color {
    pub r: Float,
    pub g: Float,
    pub b: Float,
    pub a: Float,
}

impl Color {
    pub const WHITE: Self = Self { r: 1.0, g: 1.0, b: 1.0, a: 1.0 };
    pub const BLACK: Self = Self { r: 0.0, g: 0.0, b: 0.0, a: 1.0 };
    pub const RED: Self = Self { r: 1.0, g: 0.0, b: 0.0, a: 1.0 };
    pub const GREEN: Self = Self { r: 0.0, g: 1.0, b: 0.0, a: 1.0 };
    pub const BLUE: Self = Self { r: 0.0, g: 0.0, b: 1.0, a: 1.0 };
    pub const TRANSPARENT: Self = Self { r: 0.0, g: 0.0, b: 0.0, a: 0.0 };

    pub fn new(r: Float, g: Float, b: Float, a: Float) -> Self {
        Self { r, g, b, a }
    }

    pub fn to_array(&self) -> ColorRGBA {
        [self.r, self.g, self.b, self.a]
    }

    pub fn from_array(array: ColorRGBA) -> Self {
        Self {
            r: array[0],
            g: array[1],
            b: array[2],
            a: array[3],
        }
    }
}

/// Renderer capabilities reported by the backend
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RendererCapabilities {
    pub max_texture_size: UInt,
    pub max_render_targets: UInt,
    pub max_color_attachments: UInt,
    pub max_vertex_attributes: UInt,
    pub max_uniform_buffer_size: UInt,
    pub max_texture_units: UInt,
    pub max_compute_work_groups: Vec3,
    pub supports_compute: bool,
    pub supports_tessellation: bool,
    pub supports_geometry_shaders: bool,
    pub supports_indirect: bool,
    pub supports_instancing: bool,
    pub supports_timestamps: bool,
    pub supports_anisotropic_filtering: bool,
    pub supports_depth_texture: bool,
    pub supports_multisample: bool,
    pub max_sample_count: UInt,
}

impl Default for RendererCapabilities {
    fn default() -> Self {
        Self {
            max_texture_size: 2048,
            max_render_targets: 4,
            max_color_attachments: 4,
            max_vertex_attributes: 16,
            max_uniform_buffer_size: 65536,
            max_texture_units: 16,
            max_compute_work_groups: [65535.0, 65535.0, 65535.0],
            supports_compute: false,
            supports_tessellation: false,
            supports_geometry_shaders: false,
            supports_indirect: false,
            supports_instancing: true,
            supports_timestamps: false,
            supports_anisotropic_filtering: true,
            supports_depth_texture: true,
            supports_multisample: true,
            max_sample_count: 4,
        }
    }
}

/// Renderer configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RendererConfig {
    pub vsync: bool,
    pub multisampling: UInt,
    pub anisotropic_filtering: Float,
    pub depth_bits: UInt,
    pub stencil_bits: UInt,
    pub srgb: bool,
    pub debug_mode: bool,
    pub power_preference: PowerPreference,
}

impl Default for RendererConfig {
    fn default() -> Self {
        Self {
            vsync: true,
            multisampling: 1,
            anisotropic_filtering: 1.0,
            depth_bits: 24,
            stencil_bits: 8,
            srgb: true,
            debug_mode: false,
            power_preference: PowerPreference::HighPerformance,
        }
    }
}

/// Power preference for GPU selection
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum PowerPreference {
    Default,
    LowPower,
    HighPerformance,
}

/// Renderer statistics
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RendererStats {
    pub frames_rendered: u64,
    pub draw_calls: u64,
    pub draw_calls_per_frame: UInt,
    pub triangles_rendered: u64,
    pub triangles_per_frame: UInt,
    pub state_changes: u64,
    pub state_changes_per_frame: UInt,
    pub resource_memory: usize,
    pub last_frame_time_ms: Float,
    pub average_frame_time_ms: Float,
    pub min_frame_time_ms: Float,
    pub max_frame_time_ms: Float,
}