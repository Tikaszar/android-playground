use crate::packet::Packet;
use std::collections::BinaryHeap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio::time::{Duration, Instant};
use bytes::{Bytes, BytesMut, BufMut};

#[derive(Clone)]
struct QueuedPacket {
    packet: Packet,
    timestamp: Instant,
}

impl PartialEq for QueuedPacket {
    fn eq(&self, other: &Self) -> bool {
        self.packet.priority == other.packet.priority &&
        self.timestamp == other.timestamp
    }
}

impl Eq for QueuedPacket {}

impl PartialOrd for QueuedPacket {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for QueuedPacket {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.packet.priority.cmp(&other.packet.priority)
            .then_with(|| other.timestamp.cmp(&self.timestamp))
    }
}

pub struct FrameBatcher {
    queues: Arc<RwLock<Vec<BinaryHeap<QueuedPacket>>>>,
    frame_rate: u32,
    max_batch_size: usize,
}

impl FrameBatcher {
    pub fn new(num_channels: usize, frame_rate: u32) -> Self {
        let mut queues = Vec::with_capacity(num_channels);
        for _ in 0..num_channels {
            queues.push(BinaryHeap::new());
        }
        
        Self {
            queues: Arc::new(RwLock::new(queues)),
            frame_rate,
            max_batch_size: 1024 * 64, // 64KB default
        }
    }
    
    pub async fn queue_packet(&self, packet: Packet) {
        let channel_id = packet.channel_id as usize;
        let mut queues = self.queues.write().await;
        
        if channel_id >= queues.len() {
            queues.resize_with(channel_id + 1, BinaryHeap::new);
        }
        
        queues[channel_id].push(QueuedPacket {
            packet,
            timestamp: Instant::now(),
        });
    }
    
    pub async fn get_batch(&self, channel_id: u16) -> Option<Bytes> {
        let mut queues = self.queues.write().await;
        let channel_idx = channel_id as usize;
        
        if channel_idx >= queues.len() {
            return None;
        }
        
        let queue = &mut queues[channel_idx];
        if queue.is_empty() {
            return None;
        }
        
        let mut batch = BytesMut::new();
        let mut batch_size = 0;
        let mut packets = Vec::new();
        
        // IMPORTANT: Don't pop() - just peek and collect packets
        // We'll clear them separately after all clients have received them
        let temp_queue: Vec<_> = queue.drain().collect();
        for queued in &temp_queue {
            let serialized = queued.packet.serialize();
            let packet_size = serialized.len() + 4;
            
            if batch_size + packet_size > self.max_batch_size && !packets.is_empty() {
                // Put back the remaining packets
                for q in temp_queue.into_iter().skip(packets.len()) {
                    queue.push(q);
                }
                break;
            }
            
            packets.push(serialized);
            batch_size += packet_size;
        }
        
        if packets.is_empty() {
            return None;
        }
        
        batch.put_u32(packets.len() as u32);
        for packet in packets {
            batch.put_u32(packet.len() as u32);
            batch.put(packet);
        }
        
        Some(batch.freeze())
    }
    
    pub async fn get_all_batches(&self) -> Vec<(u16, Bytes)> {
        let mut result = Vec::new();
        let queue_count = {
            let queues = self.queues.read().await;
            queues.len()
        };
        
        for channel_id in 0..queue_count {
            if let Some(batch) = self.get_batch(channel_id as u16).await {
                result.push((channel_id as u16, batch));
            }
        }
        
        result
    }
    
    pub fn frame_duration(&self) -> Duration {
        Duration::from_millis(1000 / self.frame_rate as u64)
    }
}