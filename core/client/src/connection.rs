use crate::packet::{Packet, Priority, ControlMessageType};
use bytes::{Bytes, BytesMut, BufMut};
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{WebSocket, MessageEvent, ErrorEvent, CloseEvent, BinaryType};
use std::cell::RefCell;
use std::rc::Rc;
use futures::channel::mpsc;
use futures::StreamExt;

pub struct WebSocketClient {
    url: String,
    socket: Rc<RefCell<Option<WebSocket>>>,
    sender: mpsc::UnboundedSender<Packet>,
    receiver: Rc<RefCell<mpsc::UnboundedReceiver<Packet>>>,
}

impl WebSocketClient {
    pub fn new(url: &str) -> Result<Self, String> {
        let (sender, receiver) = mpsc::unbounded();
        
        Ok(Self {
            url: url.to_string(),
            socket: Rc::new(RefCell::new(None)),
            sender,
            receiver: Rc::new(RefCell::new(receiver)),
        })
    }
    
    pub async fn connect(&mut self) -> Result<(), String> {
        let ws = WebSocket::new(&self.url)
            .map_err(|e| format!("Failed to create WebSocket: {:?}", e))?;
        
        ws.set_binary_type(BinaryType::Arraybuffer);
        
        let socket_clone = self.socket.clone();
        let onopen = Closure::wrap(Box::new(move || {
            web_sys::console::log_1(&"WebSocket connected".into());
        }) as Box<dyn FnMut()>);
        
        let onmessage = Closure::wrap(Box::new(move |e: MessageEvent| {
            if let Ok(array_buffer) = e.data().dyn_into::<js_sys::ArrayBuffer>() {
                let array = js_sys::Uint8Array::new(&array_buffer);
                let data = array.to_vec();
                
                match Packet::deserialize(Bytes::from(data)) {
                    Ok(packet) => {
                        web_sys::console::log_1(&format!("Received packet on channel {}", packet.channel_id).into());
                    }
                    Err(e) => {
                        web_sys::console::error_1(&format!("Failed to deserialize packet: {}", e).into());
                    }
                }
            }
        }) as Box<dyn FnMut(MessageEvent)>);
        
        let onerror = Closure::wrap(Box::new(move |e: ErrorEvent| {
            web_sys::console::error_1(&format!("WebSocket error: {:?}", e).into());
        }) as Box<dyn FnMut(ErrorEvent)>);
        
        let onclose = Closure::wrap(Box::new(move |e: CloseEvent| {
            web_sys::console::log_1(&format!("WebSocket closed: code={} reason={}", 
                e.code(), e.reason()).into());
        }) as Box<dyn FnMut(CloseEvent)>);
        
        ws.set_onopen(Some(onopen.as_ref().unchecked_ref()));
        ws.set_onmessage(Some(onmessage.as_ref().unchecked_ref()));
        ws.set_onerror(Some(onerror.as_ref().unchecked_ref()));
        ws.set_onclose(Some(onclose.as_ref().unchecked_ref()));
        
        onopen.forget();
        onmessage.forget();
        onerror.forget();
        onclose.forget();
        
        *self.socket.borrow_mut() = Some(ws);
        
        self.start_send_loop();
        
        Ok(())
    }
    
    fn start_send_loop(&self) {
        let socket = self.socket.clone();
        let mut receiver = self.receiver.borrow_mut();
        
        wasm_bindgen_futures::spawn_local(async move {
            while let Some(packet) = receiver.next().await {
                if let Some(ws) = socket.borrow().as_ref() {
                    let data = packet.serialize();
                    let array = js_sys::Uint8Array::from(data.as_ref());
                    
                    if let Err(e) = ws.send_with_u8_array(&array) {
                        web_sys::console::error_1(&format!("Failed to send packet: {:?}", e).into());
                    }
                }
            }
        });
    }
    
    pub async fn send_packet(&self, packet: Packet) -> Result<(), String> {
        self.sender.unbounded_send(packet)
            .map_err(|e| format!("Failed to queue packet: {}", e))
    }
    
    pub async fn send_control_register_system(&self, name: &str, channel_id: u16) -> Result<(), String> {
        let payload = format!("{}:{}", name, channel_id);
        let packet = Packet::new(
            0,
            ControlMessageType::RegisterSystem as u16,
            Priority::High,
            Bytes::from(payload.into_bytes()),
        );
        
        self.send_packet(packet).await
    }
    
    pub async fn send_control_register_plugin(&self, name: &str) -> Result<(), String> {
        let packet = Packet::new(
            0,
            ControlMessageType::RegisterPlugin as u16,
            Priority::High,
            Bytes::from(name.as_bytes().to_vec()),
        );
        
        self.send_packet(packet).await
    }
    
    pub fn is_connected(&self) -> bool {
        self.socket.borrow()
            .as_ref()
            .map(|ws| ws.ready_state() == WebSocket::OPEN)
            .unwrap_or(false)
    }
}