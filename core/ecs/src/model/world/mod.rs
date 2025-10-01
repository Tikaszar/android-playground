//! World module - EXPORTS ONLY

pub mod world;
pub mod world_ref;
pub mod world_stats;
pub mod world_metadata;

// Re-exports
pub use world::World;
pub use world_ref::WorldRef;
pub use world_stats::WorldStats;
pub use world_metadata::WorldMetadata;