use std::sync::Arc;
use std::collections::BinaryHeap;
use std::cmp::Ordering;
use std::time::Duration;
use async_trait::async_trait;
use playground_core_server::{BatcherContract, Packet, Priority};
use playground_core_types::{Shared, shared};

/// Wrapper for packets in the priority queue
#[derive(Clone)]
struct PrioritizedPacket {
    packet: Packet,
    sequence: usize,
}

impl PartialEq for PrioritizedPacket {
    fn eq(&self, other: &Self) -> bool {
        self.packet.priority == other.packet.priority && self.sequence == other.sequence
    }
}

impl Eq for PrioritizedPacket {}

impl PartialOrd for PrioritizedPacket {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for PrioritizedPacket {
    fn cmp(&self, other: &Self) -> Ordering {
        // Higher priority first, then by sequence
        match other.packet.priority.cmp(&self.packet.priority) {
            Ordering::Equal => self.sequence.cmp(&other.sequence),
            other => other,
        }
    }
}

pub struct FrameBatcher {
    queue: Shared<BinaryHeap<PrioritizedPacket>>,
    sequence_counter: Shared<usize>,
    frame_duration_ms: u64,
}

impl FrameBatcher {
    pub fn new(fps: u32) -> Self {
        let frame_duration_ms = 1000 / fps as u64;
        
        Self {
            queue: shared(BinaryHeap::new()),
            sequence_counter: shared(0),
            frame_duration_ms,
        }
    }
}

#[async_trait]
impl BatcherContract for FrameBatcher {
    async fn queue_packet(&self, packet: Packet) {
        // Blocker priority packets bypass the queue
        if matches!(packet.priority, Priority::Blocker) {
            // These should be sent immediately by the caller
            return;
        }
        
        let mut queue = self.queue.write().await;
        let mut seq = self.sequence_counter.write().await;
        
        queue.push(PrioritizedPacket {
            packet,
            sequence: *seq,
        });
        
        *seq = seq.wrapping_add(1);
    }
    
    async fn get_batch(&self) -> Vec<Packet> {
        let mut queue = self.queue.write().await;
        let mut batch = Vec::new();
        
        // Collect up to 100 packets or until queue is empty
        const MAX_BATCH_SIZE: usize = 100;
        
        while !queue.is_empty() && batch.len() < MAX_BATCH_SIZE {
            if let Some(prioritized) = queue.pop() {
                batch.push(prioritized.packet);
            }
        }
        
        batch
    }
    
    fn frame_duration(&self) -> Duration {
        Duration::from_millis(self.frame_duration_ms)
    }
    
    fn set_frame_rate(&mut self, fps: u32) {
        self.frame_duration_ms = 1000 / fps as u64;
    }
    
    async fn start_batch_loop(self: Arc<Self>) {
        let mut interval = tokio::time::interval(self.frame_duration());
        
        loop {
            interval.tick().await;
            
            let batch = self.get_batch().await;
            if !batch.is_empty() {
                // In a real implementation, this would send the batch
                // For now, we just collect them
                // The actual sending would be done by the WebSocket handler
            }
        }
    }
}