//! Shared components used by both 2D and 3D rendering

pub mod camera;
pub mod visibility;
pub mod render_layer;
pub mod light;

#[cfg(feature = "textures")]
pub mod texture;

#[cfg(feature = "shaders")]
pub mod shader;

#[cfg(feature = "shaders")]
pub mod material;

// Always available components
pub use camera::Camera;
pub use visibility::Visibility;
pub use render_layer::RenderLayer;
pub use light::{Light, LightType};

// Feature-gated components
#[cfg(feature = "textures")]
pub use texture::{Texture, TextureFormat};

#[cfg(feature = "shaders")]
pub use shader::{Shader, ShaderType};

#[cfg(feature = "shaders")]
pub use material::{Material, UniformValue, BlendMode, CullMode};