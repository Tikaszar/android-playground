use std::time::{Duration, Instant, SystemTime};
use std::collections::{HashMap, VecDeque};
use std::io::{self, Write};
use tokio::fs::OpenOptions;
use tokio::io::AsyncWriteExt;
use chrono;
use playground_core_types::{Handle, Shared, shared};

#[derive(Clone, Debug)]
pub struct ClientInfo {
    pub id: usize,
    pub connected_at: Instant,
    pub last_activity: Instant,
    pub messages_sent: u64,
    pub messages_received: u64,
    pub bytes_sent: u64,
    pub bytes_received: u64,
    pub ip_address: String,
    pub user_agent: Option<String>,
    pub status: ClientStatus,
}

#[derive(Clone, Debug, PartialEq)]
pub enum ClientStatus {
    Connecting,
    Connected,
    Idle,
    Active,
    Disconnecting,
    Disconnected,
}

impl ClientStatus {
    pub fn as_emoji(&self) -> &str {
        match self {
            ClientStatus::Connecting => "🔄",
            ClientStatus::Connected => "✅",
            ClientStatus::Idle => "💤",
            ClientStatus::Active => "🟢",
            ClientStatus::Disconnecting => "🔻",
            ClientStatus::Disconnected => "❌",
        }
    }
    
    pub fn as_color_code(&self) -> &str {
        match self {
            ClientStatus::Connecting => "\x1b[33m",    // Yellow
            ClientStatus::Connected => "\x1b[32m",      // Green
            ClientStatus::Idle => "\x1b[90m",           // Gray
            ClientStatus::Active => "\x1b[92m",         // Bright Green
            ClientStatus::Disconnecting => "\x1b[93m",  // Bright Yellow
            ClientStatus::Disconnected => "\x1b[31m",   // Red
        }
    }
}

#[derive(Clone, Debug)]
pub struct LogEntry {
    pub timestamp: SystemTime,
    pub level: LogLevel,
    pub message: String,
    pub client_id: Option<usize>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum LogLevel {
    Info,
    Warning,
    Error,
    Debug,
}

impl LogLevel {
    pub fn as_emoji(&self) -> &str {
        match self {
            LogLevel::Info => "ℹ️",
            LogLevel::Warning => "⚠️",
            LogLevel::Error => "❌",
            LogLevel::Debug => "🔍",
        }
    }
    
    pub fn as_color_code(&self) -> &str {
        match self {
            LogLevel::Info => "\x1b[36m",     // Cyan
            LogLevel::Warning => "\x1b[33m",  // Yellow
            LogLevel::Error => "\x1b[31m",    // Red
            LogLevel::Debug => "\x1b[90m",    // Gray
        }
    }
}

pub struct Dashboard {
    clients: Shared<HashMap<usize, ClientInfo>>,
    temp_clients: Shared<HashMap<usize, ClientInfo>>,  // Unverified connections
    server_start: Instant,
    total_connections: Shared<u64>,
    total_messages: Shared<u64>,
    total_bytes: Shared<u64>,
    mcp_sessions: Shared<HashMap<String, Instant>>,
    last_update: Shared<Instant>,
    recent_logs: Shared<VecDeque<LogEntry>>,
    log_file: Shared<Option<tokio::fs::File>>,
    max_log_entries: usize,
}

impl Dashboard {
    pub fn new() -> Self {
        Self {
            clients: shared(HashMap::new()),
            temp_clients: shared(HashMap::new()),
            server_start: Instant::now(),
            total_connections: shared(0),
            total_messages: shared(0),
            total_bytes: shared(0),
            mcp_sessions: shared(HashMap::new()),
            last_update: shared(Instant::now()),
            recent_logs: shared(VecDeque::with_capacity(100)),
            log_file: shared(None),
            max_log_entries: 10, // Show last 10 logs in dashboard
        }
    }
    
    /// Initialize log file
    pub async fn init_log_file(&self) -> Result<(), std::io::Error> {
        let timestamp = chrono::Local::now().format("%Y%m%d_%H%M%S");
        let log_path = format!("logs/playground_server_{}.log", timestamp);
        
        // Create logs directory if it doesn't exist
        tokio::fs::create_dir_all("logs").await?;
        
        let file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&log_path)
            .await?;
        
        // Store the file - use a scope to drop the lock before calling log()
        {
            let mut log_file = self.log_file.write().await;
            *log_file = Some(file);
        } // Lock is dropped here
        
        // Log the startup - now the lock is free
        self.log(LogLevel::Info, format!("Server started - Log file: {}", log_path), None).await;
        
        Ok(())
    }
    
    /// Add a log entry
    pub async fn log(&self, level: LogLevel, message: String, client_id: Option<usize>) {
        let entry = LogEntry {
            timestamp: SystemTime::now(),
            level: level.clone(),
            message: message.clone(),
            client_id,
        };
        
        // Add to recent logs (for dashboard display)
        let mut logs = self.recent_logs.write().await;
        if logs.len() >= 100 {
            logs.pop_front();
        }
        logs.push_back(entry.clone());
        drop(logs); // Explicitly drop the lock
        
        // Write to log file
        let mut log_file_lock = self.log_file.write().await;
        if let Some(file) = &mut *log_file_lock {
            let timestamp = chrono::Local::now().format("%Y-%m-%d %H:%M:%S%.3f");
            let client_str = client_id.map_or(String::new(), |id| format!(" [Client #{}]", id));
            let log_line = format!("[{}] {:?}{}: {}\n", timestamp, level, client_str, message);
            
            let _ = file.write_all(log_line.as_bytes()).await;
            let _ = file.flush().await;
        }
    }
    
    pub async fn add_client(&self, id: usize, ip: String) -> ClientInfo {
        let mut temp_clients = self.temp_clients.write().await;
        let mut total = self.total_connections.write().await;
        *total += 1;
        
        let info = ClientInfo {
            id,
            connected_at: Instant::now(),
            last_activity: Instant::now(),
            messages_sent: 0,
            messages_received: 0,
            bytes_sent: 0,
            bytes_received: 0,
            ip_address: ip.clone(),
            user_agent: None,
            status: ClientStatus::Connecting,  // Start as connecting
        };
        
        // Add to temp_clients first (unverified)
        temp_clients.insert(id, info.clone());
        
        // Log the connection attempt
        self.log(
            LogLevel::Info,
            format!("Client connecting from {} (unverified)", ip),
            Some(id)
        ).await;
        
        info
    }
    
    /// Verify a client after first valid message
    pub async fn verify_client(&self, id: usize) {
        let mut temp_clients = self.temp_clients.write().await;
        let mut clients = self.clients.write().await;
        
        if let Some(mut client) = temp_clients.remove(&id) {
            client.status = ClientStatus::Connected;
            clients.insert(id, client);
            
            self.log(
                LogLevel::Info,
                format!("Client verified and connected"),
                Some(id)
            ).await;
        }
    }
    
    pub async fn remove_client(&self, id: usize) {
        // Try to remove from both temp and verified clients
        let mut temp_clients = self.temp_clients.write().await;
        let mut clients = self.clients.write().await;
        
        let was_temp = temp_clients.remove(&id).is_some();
        let was_verified = clients.remove(&id).is_some();
        
        if was_temp || was_verified {
            // Log the disconnection
            self.log(
                LogLevel::Info,
                format!("Client disconnected ({})", 
                    if was_verified { "verified" } else { "unverified" }
                ),
                Some(id)
            ).await;
        }
    }
    
    pub async fn update_client_activity(&self, id: usize, received: bool, bytes: u64) {
        // First check if this is an unverified client's first activity
        {
            let temp_clients = self.temp_clients.read().await;
            if temp_clients.contains_key(&id) {
                drop(temp_clients);
                // Verify the client on first activity
                self.verify_client(id).await;
            }
        }
        
        let mut clients = self.clients.write().await;
        let mut total_msgs = self.total_messages.write().await;
        let mut total_bytes = self.total_bytes.write().await;
        
        if let Some(client) = clients.get_mut(&id) {
            client.last_activity = Instant::now();
            client.status = ClientStatus::Active;
            
            if received {
                client.messages_received += 1;
                client.bytes_received += bytes;
            } else {
                client.messages_sent += 1;
                client.bytes_sent += bytes;
            }
            
            *total_msgs += 1;
            *total_bytes += bytes;
        }
    }
    
    pub async fn add_mcp_session(&self, session_id: String) {
        let mut sessions = self.mcp_sessions.write().await;
        sessions.insert(session_id.clone(), Instant::now());
        
        // Log MCP session
        self.log(
            LogLevel::Info,
            format!("MCP session established: {}", &session_id[..20.min(session_id.len())]),
            None
        ).await;
    }
    
    pub async fn remove_mcp_session(&self, session_id: &str) {
        let mut sessions = self.mcp_sessions.write().await;
        sessions.remove(session_id);
        
        // Log MCP disconnection
        self.log(
            LogLevel::Info,
            format!("MCP session ended: {}", &session_id[..20.min(session_id.len())]),
            None
        ).await;
    }
    
    /// Log an error with client context
    pub async fn log_error(&self, message: String, client_id: Option<usize>) {
        self.log(LogLevel::Error, message, client_id).await;
    }
    
    /// Log a warning with client context
    pub async fn log_warning(&self, message: String, client_id: Option<usize>) {
        self.log(LogLevel::Warning, message, client_id).await;
    }
    
    /// Log debug information
    pub async fn log_debug(&self, message: String, client_id: Option<usize>) {
        self.log(LogLevel::Debug, message, client_id).await;
    }
    
    pub async fn render(&self) {
        // Clear screen and move cursor to top
        print!("\x1b[2J\x1b[H");
        let _ = io::stdout().flush();
        
        let temp_clients = self.temp_clients.read().await;
        let clients = self.clients.read().await;
        let total_conns = self.total_connections.read().await;
        let total_msgs = self.total_messages.read().await;
        let total_bytes = self.total_bytes.read().await;
        let mcp_sessions = self.mcp_sessions.read().await;
        
        let uptime = self.server_start.elapsed();
        let active_clients = clients.len() + temp_clients.len();
        
        // Header
        println!("\x1b[1;36m╔════════════════════════════════════════════════════════════════════╗\x1b[0m");
        println!("\x1b[1;36m║       🚀 ANDROID PLAYGROUND SERVER DASHBOARD 🚀                    ║\x1b[0m");
        println!("\x1b[1;36m╚════════════════════════════════════════════════════════════════════╝\x1b[0m");
        println!();
        
        // Server Stats
        println!("\x1b[1;33m📊 Server Statistics\x1b[0m");
        println!("├─ Uptime: \x1b[32m{}\x1b[0m", format_duration(uptime));
        println!("├─ Total Connections: \x1b[32m{}\x1b[0m", total_conns);
        println!("├─ Active Clients: \x1b[32m{}/{}\x1b[0m", active_clients, clients.len());
        println!("├─ MCP Sessions: \x1b[32m{}\x1b[0m", mcp_sessions.len());
        println!("├─ Total Messages: \x1b[32m{}\x1b[0m", format_number(*total_msgs));
        println!("└─ Total Data: \x1b[32m{}\x1b[0m", format_bytes(*total_bytes));
        println!();
        
        // MCP Sessions
        if !mcp_sessions.is_empty() {
            println!("\x1b[1;35m🤖 MCP Sessions\x1b[0m");
            for (session_id, connected_at) in mcp_sessions.iter() {
                let duration = connected_at.elapsed();
                let short_id = if session_id.len() > 20 {
                    &session_id[..20]
                } else {
                    session_id
                };
                println!("├─ {} \x1b[90m({})\x1b[0m", short_id, format_duration(duration));
            }
            println!();
        }
        
        // Client Connections
        println!("\x1b[1;34m🔌 Client Connections\x1b[0m");
        
        // Combine temp and verified clients for display
        let mut all_clients: Vec<_> = Vec::new();
        for client in temp_clients.values() {
            all_clients.push((client, false)); // false = unverified
        }
        for client in clients.values() {
            all_clients.push((client, true)); // true = verified
        }
        
        if all_clients.is_empty() {
            println!("└─ \x1b[90mNo clients connected\x1b[0m");
        } else {
            all_clients.sort_by_key(|(c, _)| c.id);
            
            for (i, (client, is_verified)) in all_clients.iter().enumerate() {
                let is_last = i == all_clients.len() - 1;
                let prefix = if is_last { "└─" } else { "├─" };
                
                let duration = client.connected_at.elapsed();
                let idle_time = client.last_activity.elapsed();
                
                // Update status based on idle time
                let status = if !is_verified {
                    ClientStatus::Connecting
                } else if idle_time > Duration::from_secs(30) {
                    ClientStatus::Idle
                } else {
                    ClientStatus::Active
                };
                
                let verification_marker = if !is_verified { " ⚠️" } else { "" };
                
                println!("{} {} Client #{:02}{} {}{}[{}]\x1b[0m",
                    prefix,
                    status.as_emoji(),
                    client.id,
                    verification_marker,
                    status.as_color_code(),
                    client.ip_address,
                    format_duration(duration)
                );
                
                let sub_prefix = if is_last { "  " } else { "│ " };
                println!("{}  📤 {} msgs / {} | 📥 {} msgs / {}",
                    sub_prefix,
                    format_number(client.messages_sent),
                    format_bytes(client.bytes_sent),
                    format_number(client.messages_received),
                    format_bytes(client.bytes_received)
                );
                
                if idle_time < Duration::from_secs(60) {
                    println!("{}  ⏱️  Last active: {}s ago",
                        sub_prefix,
                        idle_time.as_secs()
                    );
                }
            }
        }
        
        // Recent Logs Section
        println!();
        println!("\x1b[1;32m📝 Recent Activity\x1b[0m");
        
        let logs = self.recent_logs.read().await;
        let recent: Vec<_> = logs.iter()
            .rev()
            .take(self.max_log_entries)
            .collect();
        
        if recent.is_empty() {
            println!("└─ \x1b[90mNo recent activity\x1b[0m");
        } else {
            for (i, entry) in recent.iter().enumerate() {
                let is_last = i == recent.len() - 1;
                let prefix = if is_last { "└─" } else { "├─" };
                
                // Format timestamp
                let duration = SystemTime::now()
                    .duration_since(entry.timestamp)
                    .unwrap_or_default();
                let time_str = if duration.as_secs() < 60 {
                    format!("[{}s]", duration.as_secs())
                } else if duration.as_secs() < 3600 {
                    format!("[{}m]", duration.as_secs() / 60)
                } else {
                    format!("[{}h]", duration.as_secs() / 3600)
                };
                
                // Format client ID if present
                let client_str = entry.client_id
                    .map(|id| format!("[#{}]", id))
                    .unwrap_or_else(|| "    ".to_string());
                
                // Truncate message if too long
                let msg = if entry.message.len() > 45 {
                    format!("{}...", &entry.message[..42])
                } else {
                    entry.message.clone()
                };
                
                // Center single-width emojis, left-align info emoji
                let emoji_with_spacing = if entry.level == LogLevel::Info {
                    format!("{}   ", entry.level.as_emoji()) // Info emoji left-aligned with 3 spaces after
                } else {
                    format!(" {} ", entry.level.as_emoji())  // Other emojis centered with space before and after
                };
                
                println!("{} {} {:>6} {:>5} {}{}\x1b[0m",
                    prefix,
                    emoji_with_spacing,
                    time_str,
                    client_str,
                    entry.level.as_color_code(),
                    msg
                );
            }
        }
        
        println!();
        println!("\x1b[90m─────────────────────────────────────────────────────────────────────\x1b[0m");
        
        // Get log file path if exists
        let log_file = self.log_file.read().await;
        if log_file.is_some() {
            let timestamp = chrono::Local::now().format("%Y%m%d");
            println!("\x1b[90mVerbose logs: logs/playground_server_{}_*.log\x1b[0m", timestamp);
        }
        
        println!("\x1b[90mPress Ctrl+C to stop server | Dashboard updates every second\x1b[0m");
        
        // Flush output
        let _ = io::stdout().flush();
    }
    
    pub async fn start_render_loop(self: Handle<Self>) {
        // Render immediately first
        self.render().await;
        
        let dashboard = self.clone();
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(1));
            let mut cleanup_counter = 0u32;
            
            loop {
                interval.tick().await;
                dashboard.render().await;
                cleanup_counter += 1;
                
                // Clean up idle clients every 30 seconds
                if cleanup_counter >= 30 {
                    cleanup_counter = 0;
                    
                    // Find and remove clients idle for >5 minutes
                    let mut to_remove = Vec::new();
                    
                    // Check temp clients
                    {
                        let temp_clients = dashboard.temp_clients.read().await;
                        for (id, client) in temp_clients.iter() {
                            if client.last_activity.elapsed() > Duration::from_secs(300) {
                                to_remove.push(*id);
                            }
                        }
                    }
                    
                    // Check verified clients
                    {
                        let clients = dashboard.clients.read().await;
                        for (id, client) in clients.iter() {
                            if client.last_activity.elapsed() > Duration::from_secs(300) {
                                to_remove.push(*id);
                            }
                        }
                    }
                    
                    // Remove idle clients
                    for id in to_remove {
                        dashboard.log(
                            LogLevel::Info,
                            format!("Removing idle client (>5 min)"),
                            Some(id)
                        ).await;
                        dashboard.remove_client(id).await;
                    }
                }
                
                // Mark idle clients (every second)
                let mut clients = dashboard.clients.write().await;
                for client in clients.values_mut() {
                    let idle_time = client.last_activity.elapsed();
                    if idle_time > Duration::from_secs(30) && client.status != ClientStatus::Idle {
                        client.status = ClientStatus::Idle;
                    } else if idle_time <= Duration::from_secs(30) && client.status == ClientStatus::Idle {
                        client.status = ClientStatus::Active;
                    }
                }
            }
        });
    }
}

fn format_duration(duration: Duration) -> String {
    let total_secs = duration.as_secs();
    let hours = total_secs / 3600;
    let minutes = (total_secs % 3600) / 60;
    let seconds = total_secs % 60;
    
    if hours > 0 {
        format!("{}h {}m {}s", hours, minutes, seconds)
    } else if minutes > 0 {
        format!("{}m {}s", minutes, seconds)
    } else {
        format!("{}s", seconds)
    }
}

fn format_bytes(bytes: u64) -> String {
    const UNITS: &[&str] = &["B", "KB", "MB", "GB"];
    let mut size = bytes as f64;
    let mut unit_idx = 0;
    
    while size >= 1024.0 && unit_idx < UNITS.len() - 1 {
        size /= 1024.0;
        unit_idx += 1;
    }
    
    if unit_idx == 0 {
        format!("{} {}", bytes, UNITS[unit_idx])
    } else {
        format!("{:.2} {}", size, UNITS[unit_idx])
    }
}

fn format_number(num: u64) -> String {
    if num >= 1_000_000 {
        format!("{:.1}M", num as f64 / 1_000_000.0)
    } else if num >= 1_000 {
        format!("{:.1}K", num as f64 / 1_000.0)
    } else {
        num.to_string()
    }
}