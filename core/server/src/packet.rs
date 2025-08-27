use bytes::{Bytes, BytesMut, BufMut, Buf};
use anyhow::{Result, bail};

// Re-export Priority from core/types
pub use playground_core_types::Priority;

#[derive(Debug, Clone)]
pub struct Packet {
    pub channel_id: u16,
    pub packet_type: u16,
    pub priority: Priority,
    pub payload: Bytes,
}

impl Packet {
    pub fn new(channel_id: u16, packet_type: u16, priority: Priority, payload: Bytes) -> Self {
        Self {
            channel_id,
            packet_type,
            priority,
            payload,
        }
    }
    
    pub fn serialize(&self) -> Bytes {
        let mut buf = BytesMut::with_capacity(
            2 + // channel_id
            2 + // packet_type
            1 + // priority
            4 + // payload_size
            self.payload.len()
        );
        
        buf.put_u16(self.channel_id);
        buf.put_u16(self.packet_type);
        buf.put_u8(self.priority as u8);
        buf.put_u32(self.payload.len() as u32);
        buf.put(self.payload.clone());
        
        buf.freeze()
    }
    
    pub fn deserialize(mut data: Bytes) -> Result<Self> {
        if data.len() < 9 {
            bail!("Packet too small: {} bytes", data.len());
        }
        
        let channel_id = data.get_u16();
        let packet_type = data.get_u16();
        let priority = Priority::try_from(data.get_u8())
            .map_err(|e| anyhow::anyhow!(e))?;
        let payload_size = data.get_u32() as usize;
        
        if data.len() < payload_size {
            bail!("Payload size mismatch: expected {} bytes, got {}", 
                  payload_size, data.len());
        }
        
        let payload = data.split_to(payload_size);
        
        Ok(Self {
            channel_id,
            packet_type,
            priority,
            payload,
        })
    }
}

#[derive(Debug, Clone)]
pub struct ControlMessage {
    pub msg_type: ControlMessageType,
    pub data: Bytes,
}

#[derive(Debug, Clone, Copy)]
pub enum ControlMessageType {
    RegisterSystem = 1,
    RegisterPlugin = 2,
    QueryChannel = 3,
    ListChannels = 4,
    RegisterResponse = 5,
    QueryResponse = 6,
    ListResponse = 7,
    RequestChannelManifest = 8,  // Browser requests full manifest
    ChannelManifest = 9,         // Server sends manifest
    ChannelRegistered = 10,      // Notification of new channel
    ChannelUnregistered = 11,    // Notification of removed channel
    Error = 255,
}

impl TryFrom<u16> for ControlMessageType {
    type Error = anyhow::Error;
    
    fn try_from(value: u16) -> Result<Self> {
        match value {
            1 => Ok(ControlMessageType::RegisterSystem),
            2 => Ok(ControlMessageType::RegisterPlugin),
            3 => Ok(ControlMessageType::QueryChannel),
            4 => Ok(ControlMessageType::ListChannels),
            5 => Ok(ControlMessageType::RegisterResponse),
            6 => Ok(ControlMessageType::QueryResponse),
            7 => Ok(ControlMessageType::ListResponse),
            8 => Ok(ControlMessageType::RequestChannelManifest),
            9 => Ok(ControlMessageType::ChannelManifest),
            10 => Ok(ControlMessageType::ChannelRegistered),
            11 => Ok(ControlMessageType::ChannelUnregistered),
            255 => Ok(ControlMessageType::Error),
            _ => bail!("Invalid control message type: {}", value),
        }
    }
}