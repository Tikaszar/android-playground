use std::collections::HashMap;
use crate::message::Message;

pub struct Context {
    pub resources: HashMap<String, Box<dyn std::any::Any + Send + Sync>>,
    pub messages: Vec<Message>,
}
