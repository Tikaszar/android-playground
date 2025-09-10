//! Channel manager contract for channel registration and discovery
//!
//! This defines the contract for managing dynamic channel allocation.

use async_trait::async_trait;
use crate::types::ChannelManifest;

/// Contract for channel registration and discovery
#[async_trait]
pub trait ChannelManagerContract: Send + Sync {
    /// Register a channel
    async fn register(&self, channel: u16, name: String) -> Result<(), Box<dyn std::error::Error>>;
    
    /// Unregister a channel
    async fn unregister(&self, channel: u16) -> Result<(), Box<dyn std::error::Error>>;
    
    /// Get channel manifest for discovery
    async fn get_manifest(&self) -> ChannelManifest;
    
    /// Get channel by name
    async fn get_channel_by_name(&self, name: &str) -> Option<u16>;
    
    /// Check if channel is registered
    async fn is_registered(&self, channel: u16) -> bool;
}