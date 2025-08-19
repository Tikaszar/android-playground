//! Common types for the networking system

use playground_types::ChannelId;

/// Incoming packet from network
#[derive(Debug, Clone)]
pub struct IncomingPacket {
    pub channel: ChannelId,
    pub packet_type: u16,
    pub data: Vec<u8>,
    pub timestamp: u64,
}

/// Network statistics
#[derive(Debug, Clone)]
pub struct NetworkStats {
    pub bytes_sent: u64,
    pub bytes_received: u64,
    pub packets_sent: u64,
    pub packets_received: u64,
    pub connections_active: u32,
    pub average_latency_ms: u32,
}