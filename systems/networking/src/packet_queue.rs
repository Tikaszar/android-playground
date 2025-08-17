//! Packet queue for frame-based batching

use crate::{NetworkResult, IncomingPacket};
use playground_types::{ChannelId, Priority};
use std::collections::{HashMap, BinaryHeap, VecDeque};
use std::cmp::Ordering;
use bytes::Bytes;

/// Packet queue for batching messages per frame
pub struct PacketQueue {
    // Outgoing packets organized by channel and priority
    outgoing: HashMap<ChannelId, PriorityQueue>,
    // Incoming packets organized by channel
    incoming: HashMap<ChannelId, VecDeque<IncomingPacket>>,
    // Maximum packets to send per frame
    max_packets_per_frame: usize,
}

impl PacketQueue {
    pub fn new() -> Self {
        Self {
            outgoing: HashMap::new(),
            incoming: HashMap::new(),
            max_packets_per_frame: 100,
        }
    }
    
    /// Enqueue a packet for sending
    pub async fn enqueue(
        &mut self,
        channel: ChannelId,
        packet_type: u16,
        data: Vec<u8>,
        priority: Priority,
    ) -> NetworkResult<()> {
        let queue = self.outgoing.entry(channel).or_insert_with(PriorityQueue::new);
        
        let packet = PrioritizedPacket {
            packet_type,
            priority: priority as u8,
            data: Bytes::from(data),
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_millis() as u64,
        };
        
        queue.push(packet);
        
        // Check if queue is getting too large
        if queue.len() > 1000 {
            tracing::warn!("Packet queue for channel {} is large: {} packets", channel, queue.len());
        }
        
        Ok(())
    }
    
    /// Get incoming packets for a channel
    pub async fn get_incoming(&self, channel: ChannelId) -> NetworkResult<Vec<IncomingPacket>> {
        Ok(self.incoming
            .get(&channel)
            .map(|queue| queue.iter().cloned().collect())
            .unwrap_or_default())
    }
    
    /// Add an incoming packet
    pub async fn add_incoming(&mut self, packet: IncomingPacket) -> NetworkResult<()> {
        let queue = self.incoming.entry(packet.channel).or_insert_with(VecDeque::new);
        queue.push_back(packet);
        
        // Limit incoming queue size
        while queue.len() > 1000 {
            queue.pop_front();
        }
        
        Ok(())
    }
    
    /// Flush packets for this frame
    pub async fn flush_frame(&mut self) -> NetworkResult<HashMap<ChannelId, Vec<OutgoingPacket>>> {
        let mut result = HashMap::new();
        let mut total_packets = 0;
        
        let outgoing_count = self.outgoing.len().max(1);
        for (channel, queue) in &mut self.outgoing {
            let mut channel_packets = Vec::new();
            
            // Take up to max packets per channel per frame
            let channel_limit = self.max_packets_per_frame / outgoing_count;
            
            while let Some(packet) = queue.pop() {
                if total_packets >= self.max_packets_per_frame || channel_packets.len() >= channel_limit {
                    // Put it back for next frame
                    queue.push(packet);
                    break;
                }
                
                channel_packets.push(OutgoingPacket {
                    packet_type: packet.packet_type,
                    priority: packet.priority,
                    data: packet.data.to_vec(),
                    timestamp: packet.timestamp,
                });
                
                total_packets += 1;
            }
            
            if !channel_packets.is_empty() {
                result.insert(*channel, channel_packets);
            }
        }
        
        Ok(result)
    }
    
    /// Clear incoming packets for a channel
    pub async fn clear_incoming(&mut self, channel: ChannelId) -> NetworkResult<()> {
        self.incoming.remove(&channel);
        Ok(())
    }
}

/// Priority queue for outgoing packets
struct PriorityQueue {
    heap: BinaryHeap<PrioritizedPacket>,
}

impl PriorityQueue {
    fn new() -> Self {
        Self {
            heap: BinaryHeap::new(),
        }
    }
    
    fn push(&mut self, packet: PrioritizedPacket) {
        self.heap.push(packet);
    }
    
    fn pop(&mut self) -> Option<PrioritizedPacket> {
        self.heap.pop()
    }
    
    fn len(&self) -> usize {
        self.heap.len()
    }
}

/// Packet with priority for heap ordering
#[derive(Debug, Clone)]
struct PrioritizedPacket {
    packet_type: u16,
    priority: u8,
    data: Bytes,
    timestamp: u64,
}

impl PartialEq for PrioritizedPacket {
    fn eq(&self, other: &Self) -> bool {
        self.priority == other.priority && self.timestamp == other.timestamp
    }
}

impl Eq for PrioritizedPacket {}

impl Ord for PrioritizedPacket {
    fn cmp(&self, other: &Self) -> Ordering {
        // Higher priority first, then older timestamp first
        match self.priority.cmp(&other.priority).reverse() {
            Ordering::Equal => self.timestamp.cmp(&other.timestamp),
            other => other,
        }
    }
}

impl PartialOrd for PrioritizedPacket {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

/// Outgoing packet data
#[derive(Debug, Clone)]
pub struct OutgoingPacket {
    pub packet_type: u16,
    pub priority: u8,
    pub data: Vec<u8>,
    pub timestamp: u64,
}