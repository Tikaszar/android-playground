//! Message queue component for batching

use crate::types::*;
use playground_core_ecs::{EntityRef, impl_component_data};
use serde::{Deserialize, Serialize};
use std::time::Duration;

/// Message queue for batched sending
#[cfg(feature = "batching")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageQueue {
    #[serde(skip)]  // Can't serialize EntityRef/Message pairs
    pub messages: Vec<(EntityRef, Message)>,  // (connection, message) pairs
    pub batch_size: usize,
    pub flush_interval: Duration,
    pub last_flush: u64,  // Timestamp in seconds since UNIX epoch
}

#[cfg(feature = "batching")]
impl_component_data!(MessageQueue);

#[cfg(feature = "batching")]
impl MessageQueue {
    pub fn new(batch_size: usize, flush_interval: Duration) -> Self {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        Self {
            messages: Vec::with_capacity(batch_size),
            batch_size,
            flush_interval,
            last_flush: now,
        }
    }

    pub fn push(&mut self, connection: EntityRef, message: Message) {
        self.messages.push((connection, message));
    }

    pub fn should_flush(&self) -> bool {
        if self.messages.len() >= self.batch_size {
            return true;
        }

        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let elapsed = Duration::from_secs(now - self.last_flush);
        elapsed >= self.flush_interval
    }

    pub fn flush(&mut self) -> Vec<(EntityRef, Message)> {
        let messages = std::mem::take(&mut self.messages);

        self.last_flush = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        messages
    }
}