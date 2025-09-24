//! Client components for ECS

pub mod client_config;
pub mod client_state;
pub mod client_stats;

#[cfg(feature = "input")]
pub mod input_state;

#[cfg(feature = "rendering")]
pub mod render_target;

#[cfg(feature = "audio")]
pub mod audio_state;

// Re-export all components
pub use client_config::ClientConfigComponent;
pub use client_state::ClientStateComponent;
pub use client_stats::ClientStatsComponent;

#[cfg(feature = "input")]
pub use input_state::InputStateComponent;

#[cfg(feature = "rendering")]
pub use render_target::RenderTargetComponent;

#[cfg(feature = "audio")]
pub use audio_state::AudioStateComponent;