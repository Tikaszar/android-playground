//! Client statistics component

use crate::types::*;
use playground_core_ecs::impl_component_data;
use serde::{Deserialize, Serialize};

/// Client statistics as an ECS component
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClientStatsComponent {
    pub stats: ClientStats,
}

impl_component_data!(ClientStatsComponent);

impl ClientStatsComponent {
    pub fn new() -> Self {
        Self {
            stats: ClientStats::default(),
        }
    }

    pub fn update_fps(&mut self, fps: Float, frame_time_ms: Float) {
        self.stats.fps = fps;
        self.stats.frame_time_ms = frame_time_ms;
        self.stats.total_frames += 1;
    }

    pub fn on_message_sent(&mut self, bytes: usize) {
        self.stats.messages_sent += 1;
        self.stats.bytes_sent += bytes as u64;
    }

    pub fn on_message_received(&mut self, bytes: usize) {
        self.stats.messages_received += 1;
        self.stats.bytes_received += bytes as u64;
    }
}

impl Default for ClientStatsComponent {
    fn default() -> Self {
        Self::new()
    }
}