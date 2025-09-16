use std::collections::HashMap;
use bytes::Bytes;
use crate::message::Message;

/// Concrete wrapper for resource data, avoiding dyn/Any
pub struct Resource {
    pub data: Bytes,
    pub type_name: String,
}

impl Resource {
    pub fn new(data: Bytes, type_name: String) -> Self {
        Self { data, type_name }
    }
}

pub struct Context {
    pub resources: HashMap<String, Resource>,
    pub messages: Vec<Message>,
}

impl Context {
    pub fn new() -> Self {
        Self {
            resources: HashMap::new(),
            messages: Vec::new(),
        }
    }
}
