//! Shared components used by both 2D and 3D rendering
//! These are MANDATORY components always available

pub mod camera;
pub mod visibility;
pub mod render_layer;
pub mod light;

// MANDATORY renderer state components
pub mod renderer_config;
pub mod renderer_stats;
pub mod renderer_capabilities;
pub mod renderer_backend;

// Always available component exports
pub use camera::Camera;
pub use visibility::Visibility;
pub use render_layer::RenderLayer;
pub use light::{Light, LightType};

// Mandatory renderer state exports
pub use renderer_config::RendererConfigComponent;
pub use renderer_stats::RendererStatsComponent;
pub use renderer_capabilities::RendererCapabilitiesComponent;
pub use renderer_backend::RendererBackend;