use crate::packet::{Packet, Priority, ControlMessageType};
use crate::reconnect::{ReconnectManager, ReconnectConfig};
use bytes::Bytes;
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
    reconnect_manager: Rc<RefCell<ReconnectManager>>,
    reconnect_callbacks: Rc<RefCell<ReconnectCallbacks>>,
    auto_reconnect: Rc<RefCell<bool>>,
}

pub struct ReconnectCallbacks {
    pub on_reconnecting: Option<Box<dyn Fn(u32)>>,
    pub on_reconnected: Option<Box<dyn Fn()>>,
    pub on_reconnect_failed: Option<Box<dyn Fn(String)>>,
}

impl WebSocketClient {
    pub fn new(url: &str) -> Result<Self, String> {
        Self::with_config(url, ReconnectConfig::default())
    }
    
    pub fn with_config(url: &str, reconnect_config: ReconnectConfig) -> Result<Self, String> {
        let (sender, receiver) = mpsc::unbounded();
        
        Ok(Self {
            url: url.to_string(),
            socket: Rc::new(RefCell::new(None)),
            sender,
            receiver: Rc::new(RefCell::new(receiver)),
            reconnect_manager: Rc::new(RefCell::new(ReconnectManager::new(reconnect_config))),
            reconnect_callbacks: Rc::new(RefCell::new(ReconnectCallbacks {
                on_reconnecting: None,
                on_reconnected: None,
                on_reconnect_failed: None,
            })),
            auto_reconnect: Rc::new(RefCell::new(true)),
        })
    }
    
    pub fn set_auto_reconnect(&mut self, enabled: bool) {
        *self.auto_reconnect.borrow_mut() = enabled;
    }
    
    pub fn set_reconnect_callbacks(&mut self, callbacks: ReconnectCallbacks) {
        *self.reconnect_callbacks.borrow_mut() = callbacks;
    }
    
    pub async fn connect(&mut self) -> Result<(), String> {
        self.connect_internal().await
    }
    
    async fn connect_internal(&mut self) -> Result<(), String> {
        let ws = WebSocket::new(&self.url)
            .map_err(|e| format!("Failed to create WebSocket: {:?}", e))?;
        
        ws.set_binary_type(BinaryType::Arraybuffer);
        
        let _socket_clone = self.socket.clone();
        let reconnect_manager = self.reconnect_manager.clone();
        let reconnect_callbacks = self.reconnect_callbacks.clone();
        let onopen = Closure::wrap(Box::new(move || {
            web_sys::console::log_1(&"WebSocket connected".into());
            reconnect_manager.borrow_mut().on_connected();
            if let Some(cb) = &reconnect_callbacks.borrow().on_reconnected {
                cb();
            }
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
        
        let socket_for_close = self.socket.clone();
        let reconnect_manager_close = self.reconnect_manager.clone();
        let auto_reconnect = self.auto_reconnect.clone();
        let url_clone = self.url.clone();
        let sender_clone = self.sender.clone();
        let receiver_clone = self.receiver.clone();
        let reconnect_callbacks_close = self.reconnect_callbacks.clone();
        
        let onclose = Closure::wrap(Box::new(move |e: CloseEvent| {
            web_sys::console::log_1(&format!("WebSocket closed: code={} reason={}", 
                e.code(), e.reason()).into());
            
            *socket_for_close.borrow_mut() = None;
            
            if *auto_reconnect.borrow() && e.code() != 1000 {
                reconnect_manager_close.borrow_mut().on_disconnected();
                
                let socket_clone = socket_for_close.clone();
                let reconnect_manager_clone = reconnect_manager_close.clone();
                let url = url_clone.clone();
                let sender = sender_clone.clone();
                let receiver = receiver_clone.clone();
                let auto_reconnect_clone = auto_reconnect.clone();
                let callbacks = reconnect_callbacks_close.clone();
                
                wasm_bindgen_futures::spawn_local(async move {
                    Self::start_reconnection_loop(
                        url,
                        socket_clone,
                        sender,
                        receiver,
                        reconnect_manager_clone,
                        callbacks,
                        auto_reconnect_clone,
                    ).await;
                });
            }
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
    
    async fn start_reconnection_loop(
        url: String,
        socket: Rc<RefCell<Option<WebSocket>>>,
        _sender: mpsc::UnboundedSender<Packet>,
        _receiver: Rc<RefCell<mpsc::UnboundedReceiver<Packet>>>,
        reconnect_manager: Rc<RefCell<ReconnectManager>>,
        callbacks: Rc<RefCell<ReconnectCallbacks>>,
        auto_reconnect: Rc<RefCell<bool>>,
    ) {
        loop {
            if !*auto_reconnect.borrow() {
                break;
            }
            
            if !reconnect_manager.borrow().should_reconnect() {
                if let Some(cb) = &callbacks.borrow().on_reconnect_failed {
                    cb("Max reconnection attempts reached".to_string());
                }
                break;
            }
            
            if let Err(e) = reconnect_manager.borrow_mut().wait_before_reconnect().await {
                web_sys::console::error_1(&format!("Reconnection wait failed: {}", e).into());
                break;
            }
            
            let attempt = {
                let manager = reconnect_manager.borrow();
                match manager.state() {
                    crate::reconnect::ReconnectState::Reconnecting { attempt, .. } => *attempt,
                    _ => 0,
                }
            };
            
            if let Some(cb) = &callbacks.borrow().on_reconnecting {
                cb(attempt);
            }
            
            match Self::attempt_reconnect(&url, socket.clone()).await {
                Ok(ws) => {
                    *socket.borrow_mut() = Some(ws);
                    reconnect_manager.borrow_mut().on_connected();
                    
                    if let Some(cb) = &callbacks.borrow().on_reconnected {
                        cb();
                    }
                    
                    web_sys::console::log_1(&"Reconnection successful".into());
                    break;
                }
                Err(e) => {
                    reconnect_manager.borrow_mut().on_reconnect_failed(e.clone());
                    
                    if let Some(cb) = &callbacks.borrow().on_reconnect_failed {
                        cb(e);
                    }
                }
            }
        }
    }
    
    async fn attempt_reconnect(url: &str, _socket: Rc<RefCell<Option<WebSocket>>>) -> Result<WebSocket, String> {
        let ws = WebSocket::new(url)
            .map_err(|e| format!("Failed to create WebSocket: {:?}", e))?;
        
        ws.set_binary_type(BinaryType::Arraybuffer);
        
        let (open_sender, mut open_receiver) = mpsc::channel(1);
        let (error_sender, mut error_receiver) = mpsc::channel(1);
        
        let open_sender = Rc::new(RefCell::new(Some(open_sender)));
        let error_sender = Rc::new(RefCell::new(Some(error_sender)));
        
        let open_sender_clone = open_sender.clone();
        let onopen = Closure::wrap(Box::new(move || {
            if let Some(mut sender) = open_sender_clone.borrow_mut().take() {
                let _ = sender.try_send(());
            }
        }) as Box<dyn FnMut()>);
        
        let error_sender_clone = error_sender.clone();
        let onerror = Closure::wrap(Box::new(move |_e: ErrorEvent| {
            if let Some(mut sender) = error_sender_clone.borrow_mut().take() {
                let _ = sender.try_send(format!("WebSocket error during reconnection"));
            }
        }) as Box<dyn FnMut(ErrorEvent)>);
        
        ws.set_onopen(Some(onopen.as_ref().unchecked_ref()));
        ws.set_onerror(Some(onerror.as_ref().unchecked_ref()));
        
        futures::select! {
            _ = open_receiver.next() => {
                onopen.forget();
                onerror.forget();
                Ok(ws)
            }
            err = error_receiver.next() => {
                onopen.forget();
                onerror.forget();
                Err(err.unwrap_or_else(|| "Unknown error".to_string()))
            }
        }
    }
    
    fn start_send_loop(&self) {
        let socket = self.socket.clone();
        let receiver = self.receiver.clone();
        
        wasm_bindgen_futures::spawn_local(async move {
            loop {
                let packet = {
                    let mut rec = receiver.borrow_mut();
                    rec.next().await
                };
                
                if let Some(packet) = packet {
                    if let Some(ws) = socket.borrow().as_ref() {
                        let data = packet.serialize();
                        
                        if let Err(e) = ws.send_with_u8_array(data.as_ref()) {
                            web_sys::console::error_1(&format!("Failed to send packet: {:?}", e).into());
                        }
                    }
                } else {
                    break;
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