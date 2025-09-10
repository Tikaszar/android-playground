//! Batcher contract for frame-based packet batching
//!
//! This defines the contract for batching packets at a specific frame rate.

use std::sync::Arc;
use std::time::Duration;
use async_trait::async_trait;
use crate::types::Packet;

/// Contract for frame-based packet batching
#[async_trait]
pub trait BatcherContract: Send + Sync {
    /// Queue a packet for batching
    async fn queue_packet(&self, packet: Packet);
    
    /// Get current batch of packets
    async fn get_batch(&self) -> Vec<Packet>;
    
    /// Get frame duration
    fn frame_duration(&self) -> Duration;
    
    /// Set frame rate
    fn set_frame_rate(&mut self, fps: u32);
    
    /// Start batch processing loop
    async fn start_batch_loop(self: Arc<Self>);
}