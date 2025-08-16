mod packet;
mod channel;
mod connection;

use wasm_bindgen::prelude::*;
use web_sys::console;

#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

#[wasm_bindgen(start)]
pub fn init() {
    #[cfg(feature = "console_error_panic_hook")]
    console_error_panic_hook::set_once();
    
    console::log_1(&"Playground Client initialized".into());
}

pub use connection::WebSocketClient;
pub use channel::ChannelManager;
pub use packet::{Packet, Priority};

#[wasm_bindgen]
pub struct Client {
    connection: WebSocketClient,
    channel_manager: ChannelManager,
}

#[wasm_bindgen]
impl Client {
    #[wasm_bindgen(constructor)]
    pub fn new(url: &str) -> Result<Client, JsValue> {
        let connection = WebSocketClient::new(url)
            .map_err(|e| JsValue::from_str(&format!("Failed to create connection: {}", e)))?;
        
        let channel_manager = ChannelManager::new();
        
        Ok(Client {
            connection,
            channel_manager,
        })
    }
    
    #[wasm_bindgen]
    pub async fn connect(&mut self) -> Result<(), JsValue> {
        self.connection.connect().await
            .map_err(|e| JsValue::from_str(&format!("Connection failed: {}", e)))
    }
    
    #[wasm_bindgen]
    pub async fn register_system(&mut self, name: String, channel_id: u16) -> Result<u16, JsValue> {
        self.channel_manager.register_system(name.clone(), channel_id)
            .map_err(|e| JsValue::from_str(&format!("Registration failed: {}", e)))?;
        
        self.connection.send_control_register_system(&name, channel_id).await
            .map_err(|e| JsValue::from_str(&format!("Failed to send registration: {}", e)))?;
        
        Ok(channel_id)
    }
    
    #[wasm_bindgen]
    pub async fn register_plugin(&mut self, name: String) -> Result<u16, JsValue> {
        self.connection.send_control_register_plugin(&name).await
            .map_err(|e| JsValue::from_str(&format!("Failed to send registration: {}", e)))?;
        
        Ok(0)
    }
    
    #[wasm_bindgen]
    pub async fn send_packet(&mut self, channel_id: u16, packet_type: u16, priority: u8, data: Vec<u8>) -> Result<(), JsValue> {
        let priority = packet::Priority::try_from(priority)
            .map_err(|e| JsValue::from_str(&format!("Invalid priority: {}", e)))?;
        
        let packet = packet::Packet::new(channel_id, packet_type, priority, bytes::Bytes::from(data));
        
        self.connection.send_packet(packet).await
            .map_err(|e| JsValue::from_str(&format!("Failed to send packet: {}", e)))
    }
    
    #[wasm_bindgen]
    pub fn is_connected(&self) -> bool {
        self.connection.is_connected()
    }
}