pub mod entity;
pub mod component;
pub mod storage;
pub mod world;
pub mod query;
pub mod error;
pub mod messaging;

pub use entity::*;
pub use component::*;
pub use storage::*;
pub use world::*;
pub use query::*;
pub use error::*;
pub use messaging::{MessageBus, MessageHandler, Broadcaster, ChannelId};

// Convenience alias
pub type Result<T> = EcsResult<T>;