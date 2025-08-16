use crate::error::{UiError, UiResult};
use futures_util::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::{mpsc, RwLock};
use tokio_tungstenite::{connect_async, tungstenite::Message, WebSocketStream, MaybeTlsStream};
use tokio::net::TcpStream;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TerminalMessage {
    Input(String),
    Output(String),
    Resize { cols: u16, rows: u16 },
    Connect,
    Disconnect,
    Heartbeat,
}

pub struct WebSocketTerminal {
    url: String,
    ws_stream: Option<WebSocketStream<MaybeTlsStream<TcpStream>>>,
    output_tx: mpsc::Sender<String>,
    output_rx: Arc<RwLock<mpsc::Receiver<String>>>,
    input_tx: mpsc::Sender<String>,
    input_rx: Arc<RwLock<mpsc::Receiver<String>>>,
    is_connected: Arc<RwLock<bool>>,
}

impl WebSocketTerminal {
    pub fn new(url: String) -> (Self, mpsc::Receiver<String>) {
        let (output_tx, output_rx) = mpsc::channel(100);
        let (input_tx, input_rx) = mpsc::channel(100);
        let (external_tx, external_rx) = mpsc::channel(100);
        
        let terminal = Self {
            url,
            ws_stream: None,
            output_tx: external_tx,
            output_rx: Arc::new(RwLock::new(output_rx)),
            input_tx,
            input_rx: Arc::new(RwLock::new(input_rx)),
            is_connected: Arc::new(RwLock::new(false)),
        };
        
        (terminal, external_rx)
    }
    
    pub async fn connect(&mut self) -> UiResult<()> {
        let (ws_stream, _) = connect_async(&self.url).await
            .map_err(|e| UiError::TerminalError(format!("Failed to connect WebSocket: {}", e)))?;
        
        self.ws_stream = Some(ws_stream);
        *self.is_connected.write().await = true;
        
        self.start_read_loop();
        self.start_write_loop();
        
        Ok(())
    }
    
    pub async fn disconnect(&mut self) -> UiResult<()> {
        *self.is_connected.write().await = false;
        
        if let Some(mut stream) = self.ws_stream.take() {
            let _ = stream.close(None).await;
        }
        
        Ok(())
    }
    
    pub async fn send_input(&self, input: String) -> UiResult<()> {
        self.input_tx.send(input).await
            .map_err(|e| UiError::TerminalError(format!("Failed to send input: {}", e)))?;
        Ok(())
    }
    
    pub async fn resize(&self, cols: u16, rows: u16) -> UiResult<()> {
        let msg = TerminalMessage::Resize { cols, rows };
        let json = serde_json::to_string(&msg)
            .map_err(|e| UiError::SerializationError(format!("Failed to serialize resize: {}", e)))?;
        
        self.input_tx.send(json).await
            .map_err(|e| UiError::TerminalError(format!("Failed to send resize: {}", e)))?;
        Ok(())
    }
    
    fn start_read_loop(&mut self) {
        let output_tx = self.output_tx.clone();
        let is_connected = self.is_connected.clone();
        
        if let Some(mut stream) = self.ws_stream.take() {
            tokio::spawn(async move {
                while *is_connected.read().await {
                    match stream.next().await {
                        Some(Ok(msg)) => {
                            match msg {
                                Message::Text(text) => {
                                    if let Ok(terminal_msg) = serde_json::from_str::<TerminalMessage>(&text) {
                                        match terminal_msg {
                                            TerminalMessage::Output(output) => {
                                                let _ = output_tx.send(output).await;
                                            }
                                            _ => {}
                                        }
                                    } else {
                                        let _ = output_tx.send(text).await;
                                    }
                                }
                                Message::Binary(data) => {
                                    if let Ok(text) = String::from_utf8(data) {
                                        let _ = output_tx.send(text).await;
                                    }
                                }
                                Message::Close(_) => {
                                    *is_connected.write().await = false;
                                    break;
                                }
                                _ => {}
                            }
                        }
                        Some(Err(e)) => {
                            eprintln!("WebSocket error: {}", e);
                            *is_connected.write().await = false;
                            break;
                        }
                        None => {
                            *is_connected.write().await = false;
                            break;
                        }
                    }
                }
            });
        }
    }
    
    fn start_write_loop(&mut self) {
        let input_rx = self.input_rx.clone();
        let is_connected = self.is_connected.clone();
        
        if let Some(mut stream) = self.ws_stream.take() {
            tokio::spawn(async move {
                let mut rx = input_rx.write().await;
                
                while *is_connected.read().await {
                    if let Some(input) = rx.recv().await {
                        let msg = Message::Text(input);
                        if let Err(e) = stream.send(msg).await {
                            eprintln!("Failed to send WebSocket message: {}", e);
                            *is_connected.write().await = false;
                            break;
                        }
                    }
                }
            });
        }
    }
    
    pub async fn is_connected(&self) -> bool {
        *self.is_connected.read().await
    }
}

pub struct AnsiParser {
    buffer: String,
    in_escape: bool,
    escape_buffer: String,
}

impl AnsiParser {
    pub fn new() -> Self {
        Self {
            buffer: String::new(),
            in_escape: false,
            escape_buffer: String::new(),
        }
    }
    
    pub fn parse(&mut self, input: &str) -> Vec<TerminalOutput> {
        let mut outputs = Vec::new();
        
        for ch in input.chars() {
            if self.in_escape {
                self.escape_buffer.push(ch);
                
                if ch.is_alphabetic() || ch == '~' {
                    outputs.push(self.process_escape_sequence());
                    self.in_escape = false;
                    self.escape_buffer.clear();
                }
            } else if ch == '\x1b' {
                if !self.buffer.is_empty() {
                    outputs.push(TerminalOutput::Text(self.buffer.clone()));
                    self.buffer.clear();
                }
                self.in_escape = true;
                self.escape_buffer.push(ch);
            } else if ch == '\r' {
                continue;
            } else if ch == '\n' {
                self.buffer.push(ch);
                outputs.push(TerminalOutput::Text(self.buffer.clone()));
                self.buffer.clear();
            } else {
                self.buffer.push(ch);
            }
        }
        
        if !self.buffer.is_empty() {
            outputs.push(TerminalOutput::Text(self.buffer.clone()));
            self.buffer.clear();
        }
        
        outputs
    }
    
    fn process_escape_sequence(&self) -> TerminalOutput {
        if self.escape_buffer.starts_with("\x1b[") {
            let seq = &self.escape_buffer[2..];
            
            if seq.ends_with('m') {
                let codes = &seq[..seq.len() - 1];
                return self.parse_sgr_codes(codes);
            }
            
            if seq.ends_with('H') || seq.ends_with('f') {
                return TerminalOutput::CursorPosition(0, 0);
            }
            
            if seq.ends_with('J') {
                return TerminalOutput::ClearScreen;
            }
            
            if seq.ends_with('K') {
                return TerminalOutput::ClearLine;
            }
        }
        
        TerminalOutput::Unknown(self.escape_buffer.clone())
    }
    
    fn parse_sgr_codes(&self, codes: &str) -> TerminalOutput {
        let parts: Vec<&str> = codes.split(';').collect();
        let mut style = TerminalStyle::default();
        
        for part in parts {
            match part.parse::<u8>() {
                Ok(0) => style = TerminalStyle::default(),
                Ok(1) => style.bold = true,
                Ok(3) => style.italic = true,
                Ok(4) => style.underline = true,
                Ok(30..=37) => style.fg_color = Some(AnsiColor::from_code(part.parse::<u8>().unwrap() - 30)),
                Ok(40..=47) => style.bg_color = Some(AnsiColor::from_code(part.parse::<u8>().unwrap() - 40)),
                Ok(90..=97) => style.fg_color = Some(AnsiColor::from_code(part.parse::<u8>().unwrap() - 90 + 8)),
                Ok(100..=107) => style.bg_color = Some(AnsiColor::from_code(part.parse::<u8>().unwrap() - 100 + 8)),
                _ => {}
            }
        }
        
        TerminalOutput::Style(style)
    }
}

#[derive(Debug, Clone)]
pub enum TerminalOutput {
    Text(String),
    Style(TerminalStyle),
    CursorPosition(u16, u16),
    ClearScreen,
    ClearLine,
    Unknown(String),
}

#[derive(Debug, Clone, Default)]
pub struct TerminalStyle {
    pub bold: bool,
    pub italic: bool,
    pub underline: bool,
    pub fg_color: Option<AnsiColor>,
    pub bg_color: Option<AnsiColor>,
}

#[derive(Debug, Clone, Copy)]
pub enum AnsiColor {
    Black = 0,
    Red = 1,
    Green = 2,
    Yellow = 3,
    Blue = 4,
    Magenta = 5,
    Cyan = 6,
    White = 7,
    BrightBlack = 8,
    BrightRed = 9,
    BrightGreen = 10,
    BrightYellow = 11,
    BrightBlue = 12,
    BrightMagenta = 13,
    BrightCyan = 14,
    BrightWhite = 15,
}

impl AnsiColor {
    pub fn from_code(code: u8) -> Self {
        match code {
            0 => AnsiColor::Black,
            1 => AnsiColor::Red,
            2 => AnsiColor::Green,
            3 => AnsiColor::Yellow,
            4 => AnsiColor::Blue,
            5 => AnsiColor::Magenta,
            6 => AnsiColor::Cyan,
            7 => AnsiColor::White,
            8 => AnsiColor::BrightBlack,
            9 => AnsiColor::BrightRed,
            10 => AnsiColor::BrightGreen,
            11 => AnsiColor::BrightYellow,
            12 => AnsiColor::BrightBlue,
            13 => AnsiColor::BrightMagenta,
            14 => AnsiColor::BrightCyan,
            15 => AnsiColor::BrightWhite,
            _ => AnsiColor::White,
        }
    }
    
    pub fn to_rgba(&self) -> [f32; 4] {
        match self {
            AnsiColor::Black => [0.0, 0.0, 0.0, 1.0],
            AnsiColor::Red => [0.8, 0.0, 0.0, 1.0],
            AnsiColor::Green => [0.0, 0.8, 0.0, 1.0],
            AnsiColor::Yellow => [0.8, 0.8, 0.0, 1.0],
            AnsiColor::Blue => [0.0, 0.0, 0.8, 1.0],
            AnsiColor::Magenta => [0.8, 0.0, 0.8, 1.0],
            AnsiColor::Cyan => [0.0, 0.8, 0.8, 1.0],
            AnsiColor::White => [0.8, 0.8, 0.8, 1.0],
            AnsiColor::BrightBlack => [0.4, 0.4, 0.4, 1.0],
            AnsiColor::BrightRed => [1.0, 0.0, 0.0, 1.0],
            AnsiColor::BrightGreen => [0.0, 1.0, 0.0, 1.0],
            AnsiColor::BrightYellow => [1.0, 1.0, 0.0, 1.0],
            AnsiColor::BrightBlue => [0.0, 0.0, 1.0, 1.0],
            AnsiColor::BrightMagenta => [1.0, 0.0, 1.0, 1.0],
            AnsiColor::BrightCyan => [0.0, 1.0, 1.0, 1.0],
            AnsiColor::BrightWhite => [1.0, 1.0, 1.0, 1.0],
        }
    }
}