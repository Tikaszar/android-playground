use crate::packet::Packet;
use playground_core_types::{Shared, shared};
use std::collections::BinaryHeap;
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
    queues: Shared<Vec<BinaryHeap<QueuedPacket>>>,
    frame_rate: u32,
    max_batch_size: usize,
    broadcast_queues: Shared<Vec<Vec<QueuedPacket>>>, // Stores packets for broadcasting
}

impl FrameBatcher {
    pub fn new(num_channels: usize, frame_rate: u32) -> Self {
        let mut queues = Vec::with_capacity(num_channels);
        let mut broadcast_queues = Vec::with_capacity(num_channels);
        for _ in 0..num_channels {
            queues.push(BinaryHeap::new());
            broadcast_queues.push(Vec::new());
        }
        
        Self {
            queues: shared(queues),
            frame_rate,
            max_batch_size: 1024 * 64, // 64KB default
            broadcast_queues: shared(broadcast_queues),
        }
    }
    
    pub async fn queue_packet(&self, packet: Packet) {
        let channel_id = packet.channel_id as usize;
        let mut queues = self.queues.write().await;
        let mut broadcast_queues = self.broadcast_queues.write().await;
        
        if channel_id >= queues.len() {
            queues.resize_with(channel_id + 1, BinaryHeap::new);
            broadcast_queues.resize_with(channel_id + 1, Vec::new);
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
    
    /// Prepare broadcast batches by moving packets from queues to broadcast_queues
    pub async fn prepare_broadcast_batches(&self) {
        let mut queues = self.queues.write().await;
        let mut broadcast_queues = self.broadcast_queues.write().await;
        
        // Clear old broadcast queues first
        for queue in broadcast_queues.iter_mut() {
            queue.clear();
        }
        
        // Now move new packets from regular queues to broadcast queues
        for channel_idx in 0..queues.len() {
            let queue = &mut queues[channel_idx];
            if !queue.is_empty() {
                // Ensure broadcast_queues has enough capacity
                if channel_idx >= broadcast_queues.len() {
                    broadcast_queues.resize_with(channel_idx + 1, Vec::new);
                }
                
                // Move all packets from queue to broadcast queue
                let packets: Vec<_> = queue.drain().collect();
                broadcast_queues[channel_idx] = packets;
            }
        }
    }
    
    /// Get broadcast batch for a specific channel (doesn't drain, returns same data for all clients)
    pub async fn get_broadcast_batch(&self, channel_id: u16) -> Option<Bytes> {
        let broadcast_queues = self.broadcast_queues.read().await;
        let channel_idx = channel_id as usize;
        
        if channel_idx >= broadcast_queues.len() {
            return None;
        }
        
        let packets = &broadcast_queues[channel_idx];
        if packets.is_empty() {
            return None;
        }
        
        let mut batch = BytesMut::new();
        let mut batch_size = 0;
        let mut serialized_packets = Vec::new();
        
        for queued in packets {
            let serialized = queued.packet.serialize();
            let packet_size = serialized.len() + 4;
            
            if batch_size + packet_size > self.max_batch_size && !serialized_packets.is_empty() {
                break;
            }
            
            serialized_packets.push(serialized);
            batch_size += packet_size;
        }
        
        if serialized_packets.is_empty() {
            return None;
        }
        
        batch.put_u32(serialized_packets.len() as u32);
        for packet in serialized_packets {
            batch.put_u32(packet.len() as u32);
            batch.put(packet);
        }
        
        Some(batch.freeze())
    }
    
    /// Get all broadcast batches (for sending to all clients)
    pub async fn get_all_broadcast_batches(&self) -> Vec<(u16, Bytes)> {
        let mut result = Vec::new();
        let queue_count = {
            let broadcast_queues = self.broadcast_queues.read().await;
            broadcast_queues.len()
        };
        
        for channel_id in 0..queue_count {
            if let Some(batch) = self.get_broadcast_batch(channel_id as u16).await {
                result.push((channel_id as u16, batch));
            }
        }
        
        result
    }
    
    /// Clear broadcast queues after all clients have received them
    pub async fn clear_broadcast_queues(&self) {
        let mut broadcast_queues = self.broadcast_queues.write().await;
        for queue in broadcast_queues.iter_mut() {
            queue.clear();
        }
    }
}