use playground_core_rendering::{RenderCommand, RenderCommandBatch};
use playground_core_types::{Shared, shared};
use std::time::{Duration, Instant};

/// Manages render command batching and frame timing
pub struct RenderBatchManager {
    current_batch: Shared<RenderCommandBatch>,
    pending_commands: Shared<Vec<RenderCommand>>,
    last_flush: Instant,
    target_frame_time: Duration,
    max_commands_per_batch: usize,
}

impl RenderBatchManager {
    pub fn new() -> Self {
        Self {
            current_batch: shared(RenderCommandBatch::new(0)),
            pending_commands: shared(Vec::with_capacity(1000)),
            last_flush: Instant::now(),
            target_frame_time: Duration::from_millis(16), // ~60fps
            max_commands_per_batch: 1000,
        }
    }
    
    /// Set target frame rate
    pub fn set_target_fps(&mut self, fps: u32) {
        self.target_frame_time = Duration::from_millis(1000 / fps as u64);
    }
    
    /// Add a render command to the current batch
    pub async fn add_command(&mut self, command: RenderCommand) {
        let mut pending = self.pending_commands.write().await;
        pending.push(command);
        
        // Check if we should flush
        if pending.len() >= self.max_commands_per_batch {
            drop(pending);
            self.flush_internal().await;
        }
    }
    
    /// Add multiple commands at once
    pub async fn add_commands(&mut self, commands: Vec<RenderCommand>) {
        let mut pending = self.pending_commands.write().await;
        pending.extend(commands);
        
        // Check if we should flush
        if pending.len() >= self.max_commands_per_batch {
            drop(pending);
            self.flush_internal().await;
        }
    }
    
    /// Check if it's time to flush based on frame timing
    pub async fn should_flush(&self) -> bool {
        self.last_flush.elapsed() >= self.target_frame_time
    }
    
    /// Flush the current batch and return it
    pub async fn flush(&mut self) -> Option<RenderCommandBatch> {
        let pending = self.pending_commands.read().await;
        if pending.is_empty() {
            return None;
        }
        drop(pending);
        
        Some(self.flush_internal().await)
    }
    
    /// Internal flush implementation
    async fn flush_internal(&mut self) -> RenderCommandBatch {
        let mut pending = self.pending_commands.write().await;
        let mut batch = self.current_batch.write().await;
        
        // Move all pending commands to the batch
        for command in pending.drain(..) {
            batch.push(command);
        }
        
        // Create new batch for next frame
        let flushed_batch = std::mem::replace(&mut *batch, RenderCommandBatch::new(0));
        
        // Update timing
        self.last_flush = Instant::now();
        
        flushed_batch
    }
    
    /// Get statistics about the batch manager
    pub async fn get_stats(&self) -> BatchStats {
        let pending = self.pending_commands.read().await;
        let batch = self.current_batch.read().await;
        
        BatchStats {
            pending_commands: pending.len(),
            batch_commands: batch.commands().len(),
            time_since_flush: self.last_flush.elapsed(),
            target_frame_time: self.target_frame_time,
        }
    }
    
    /// Clear all pending commands without flushing
    pub async fn clear(&mut self) {
        self.pending_commands.write().await.clear();
        *self.current_batch.write().await = RenderCommandBatch::new(0);
    }
}

impl Default for RenderBatchManager {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone)]
pub struct BatchStats {
    pub pending_commands: usize,
    pub batch_commands: usize,
    pub time_since_flush: Duration,
    pub target_frame_time: Duration,
}

impl BatchStats {
    pub fn is_ready_to_flush(&self) -> bool {
        self.time_since_flush >= self.target_frame_time || 
        self.pending_commands > 100
    }
}