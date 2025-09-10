use std::sync::Arc;
use std::collections::{HashMap, VecDeque};
use std::time::{SystemTime, Instant};
use std::io::{self, Write};
use async_trait::async_trait;
use serde_json::Value;
use tokio::fs::OpenOptions;
use tokio::io::AsyncWriteExt;
use playground_core_server::{
    DashboardContract, LogLevel, ChannelType, ClientInfo, DashboardChannelInfo
};
use playground_core_types::{Shared, shared};
use chrono;

const MAX_LOG_ENTRIES: usize = 100;
const DASHBOARD_REFRESH_MS: u64 = 1000;

pub struct Dashboard {
    enabled: bool,
    log_entries: Shared<VecDeque<LogEntry>>,
    clients: Shared<HashMap<usize, ClientInfo>>,
    channels: Shared<HashMap<u16, DashboardChannelInfo>>,
    log_file: Shared<Option<tokio::fs::File>>,
    component_log_files: Shared<HashMap<String, tokio::fs::File>>,
    start_time: Instant,
}

#[derive(Clone, Debug)]
struct LogEntry {
    timestamp: SystemTime,
    level: LogLevel,
    message: String,
    component: Option<String>,
}

impl Dashboard {
    pub async fn new(enabled: bool) -> Result<Self, Box<dyn std::error::Error>> {
        Ok(Self {
            enabled,
            log_entries: shared(VecDeque::with_capacity(MAX_LOG_ENTRIES)),
            clients: shared(HashMap::new()),
            channels: shared(HashMap::new()),
            log_file: shared(None),
            component_log_files: shared(HashMap::new()),
            start_time: Instant::now(),
        })
    }
    
    async fn add_log_entry(&self, entry: LogEntry) {
        let mut entries = self.log_entries.write().await;
        if entries.len() >= MAX_LOG_ENTRIES {
            entries.pop_front();
        }
        entries.push_back(entry);
    }
    
    async fn write_to_log_file(&self, message: &str) {
        let log_file = self.log_file.read().await;
        if let Some(ref mut file) = log_file.as_ref() {
            let mut file = file.try_clone().await.unwrap();
            let _ = file.write_all(format!("{}\n", message).as_bytes()).await;
        }
    }
    
    async fn write_to_component_log(&self, component: &str, message: &str) {
        let mut files = self.component_log_files.write().await;
        
        if !files.contains_key(component) {
            // Create component log file
            let timestamp = chrono::Local::now().format("%Y%m%d_%H%M%S");
            let safe_component = component.replace('/', "_");
            let filename = format!("logs/playground_editor_{}_{}.log", safe_component, timestamp);
            
            if let Ok(file) = OpenOptions::new()
                .create(true)
                .append(true)
                .open(&filename)
                .await
            {
                files.insert(component.to_string(), file);
            }
        }
        
        if let Some(file) = files.get_mut(component) {
            let _ = file.write_all(format!("{}\n", message).as_bytes()).await;
        }
    }
}

#[async_trait]
impl DashboardContract for Dashboard {
    async fn log(&self, level: LogLevel, message: String, _details: Option<Value>) {
        if !self.enabled {
            return;
        }
        
        let entry = LogEntry {
            timestamp: SystemTime::now(),
            level: level.clone(),
            message: message.clone(),
            component: None,
        };
        
        let timestamp = chrono::Local::now().format("%H:%M:%S%.3f");
        let formatted = format!(
            "[{}] {} {}",
            timestamp,
            level.as_emoji(),
            message
        );
        
        self.add_log_entry(entry).await;
        self.write_to_log_file(&formatted).await;
    }
    
    async fn log_component(&self, component: &str, level: LogLevel, message: String) {
        if !self.enabled {
            return;
        }
        
        let entry = LogEntry {
            timestamp: SystemTime::now(),
            level: level.clone(),
            message: message.clone(),
            component: Some(component.to_string()),
        };
        
        let timestamp = chrono::Local::now().format("%H:%M:%S%.3f");
        let formatted = format!(
            "[{}] [{}] {} {}",
            timestamp,
            component,
            level.as_emoji(),
            message
        );
        
        self.add_log_entry(entry).await;
        self.write_to_log_file(&formatted).await;
        self.write_to_component_log(component, &formatted).await;
    }
    
    async fn register_channel(&self, id: u16, name: String, channel_type: ChannelType) {
        let mut channels = self.channels.write().await;
        channels.insert(id, DashboardChannelInfo {
            name,
            channel_id: id,
            channel_type,
            registered_at: Instant::now(),
            message_count: 0,
        });
    }
    
    async fn update_client(&self, id: usize, info: ClientInfo) {
        let mut clients = self.clients.write().await;
        clients.insert(id, info);
    }
    
    async fn init_log_file(&self) -> Result<(), std::io::Error> {
        // Ensure logs directory exists
        tokio::fs::create_dir_all("logs").await?;
        
        let timestamp = chrono::Local::now().format("%Y%m%d_%H%M%S");
        let filename = format!("logs/playground_editor_{}.log", timestamp);
        
        let file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(filename)
            .await?;
        
        let mut log_file = self.log_file.write().await;
        *log_file = Some(file);
        
        Ok(())
    }
    
    async fn start_render_loop(self: Arc<Self>) {
        if !self.enabled {
            return;
        }
        
        loop {
            tokio::time::sleep(tokio::time::Duration::from_millis(DASHBOARD_REFRESH_MS)).await;
            
            // Clear screen and render dashboard
            print!("\x1b[2J\x1b[H");
            
            // Header
            println!("┌─────────────────────────────────────────────────────────────┐");
            println!("│ 🎮 Android Playground Server Dashboard                      │");
            println!("├─────────────────────────────────────────────────────────────┤");
            
            // Server info
            let uptime = self.start_time.elapsed();
            println!("│ ⏱️  Uptime: {:02}:{:02}:{:02}                                          │",
                uptime.as_secs() / 3600,
                (uptime.as_secs() % 3600) / 60,
                uptime.as_secs() % 60
            );
            
            // Clients
            let clients = self.clients.read().await;
            println!("│ 👥 Clients: {} connected                                    │", clients.len());
            for (id, client) in clients.iter().take(3) {
                println!("│   {} Client {}: {} msgs                                │",
                    client.status.as_emoji(), id, client.messages_sent
                );
            }
            
            // Channels
            let channels = self.channels.read().await;
            println!("│ 📡 Channels: {} registered                                  │", channels.len());
            
            // Recent logs
            println!("├─────────────────────────────────────────────────────────────┤");
            println!("│ 📋 Recent Logs:                                             │");
            
            let entries = self.log_entries.read().await;
            for entry in entries.iter().rev().take(10) {
                let timestamp = chrono::DateTime::<chrono::Local>::from(entry.timestamp)
                    .format("%H:%M:%S");
                let msg = if entry.message.len() > 40 {
                    format!("{}...", &entry.message[..37])
                } else {
                    entry.message.clone()
                };
                
                println!("│ [{}] {} {}│",
                    timestamp,
                    entry.level.as_emoji(),
                    format!("{:<40}", msg)
                );
            }
            
            println!("└─────────────────────────────────────────────────────────────┘");
            
            let _ = io::stdout().flush();
        }
    }
    
    async fn get_recent_logs(&self, count: usize) -> Vec<String> {
        let entries = self.log_entries.read().await;
        entries.iter()
            .rev()
            .take(count)
            .map(|e| {
                let timestamp = chrono::DateTime::<chrono::Local>::from(e.timestamp)
                    .format("%H:%M:%S");
                format!("[{}] {} {}", timestamp, e.level.as_emoji(), e.message)
            })
            .collect()
    }
}