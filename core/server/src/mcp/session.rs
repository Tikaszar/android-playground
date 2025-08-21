use chrono::{DateTime, Utc};
use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use playground_core_types::{Shared, shared};
use tokio::sync::mpsc;
use tracing::info;
use uuid::Uuid;

use crate::mcp::protocol::{ClientInfo, McpMessage};

pub type SessionId = String;

/// Session for a connected LLM client (Claude Code, GPT, Llama, etc.)
#[derive(Debug, Clone)]
pub struct Session {
    pub id: SessionId,
    pub client_info: Option<ClientInfo>,
    pub created_at: DateTime<Utc>,
    pub last_activity: DateTime<Utc>,
    pub context_files: Vec<String>,
    pub message_tx: mpsc::UnboundedSender<McpMessage>,
}

impl Session {
    pub fn new(id: Option<String>) -> (Self, mpsc::UnboundedReceiver<McpMessage>) {
        let session_id = id.unwrap_or_else(|| Uuid::new_v4().to_string());
        let (tx, rx) = mpsc::unbounded_channel();
        
        let session = Self {
            id: session_id,
            client_info: None,
            created_at: Utc::now(),
            last_activity: Utc::now(),
            context_files: Vec::new(),
            message_tx: tx,
        };
        
        (session, rx)
    }

    pub fn update_activity(&mut self) {
        self.last_activity = Utc::now();
    }

    pub fn set_client_info(&mut self, info: ClientInfo) {
        self.client_info = Some(info);
    }

    pub fn add_context_file(&mut self, path: String) {
        if !self.context_files.contains(&path) {
            self.context_files.push(path);
        }
    }

    pub fn clear_context(&mut self) {
        self.context_files.clear();
    }

    pub fn send_message(&self, message: McpMessage) -> Result<(), String> {
        self.message_tx
            .send(message)
            .map_err(|_| "Failed to send message to session".to_string())
    }
}

/// Manages all active sessions
#[derive(Clone)]
pub struct SessionManager {
    sessions: Shared<HashMap<SessionId, Session>>,
    sse_senders: Shared<HashMap<SessionId, mpsc::UnboundedSender<Value>>>,
}

impl SessionManager {
    pub fn new() -> Self {
        Self {
            sessions: shared(HashMap::new()),
            sse_senders: shared(HashMap::new()),
        }
    }
    
    pub async fn register_sse_sender(&self, session_id: String, sender: mpsc::UnboundedSender<Value>) {
        self.sse_senders.write().await.insert(session_id, sender);
    }
    
    pub async fn get_last_sse_session(&self) -> Option<String> {
        // Get the most recently added SSE session
        self.sse_senders.read().await.iter().next().map(|(key, _)| key.clone())
    }
    
    pub async fn update_session_id(&self, old_id: &str, new_id: String) {
        // Move the SSE sender from old ID to new ID
        if let Some(sender) = self.sse_senders.write().await.remove(old_id) {
            self.sse_senders.write().await.insert(new_id.clone(), sender);
            info!("Updated session ID from {} to {}", old_id, new_id);
        }
    }
    
    pub async fn send_to_sse(&self, session_id: &str, message: Value) -> Result<(), String> {
        if let Some(sender) = self.sse_senders.read().await.get(session_id) {
            sender.send(message)
                .map_err(|_| "Failed to send SSE message".to_string())
        } else {
            Err(format!("SSE sender not found for session {}", session_id))
        }
    }

    pub async fn create_session(&self, id: Option<String>) -> (SessionId, mpsc::UnboundedReceiver<McpMessage>) {
        let (session, rx) = Session::new(id);
        let session_id = session.id.clone();
        self.sessions.write().await.insert(session_id.clone(), session);
        (session_id, rx)
    }

    pub async fn get_session(&self, id: &str) -> Option<Session> {
        self.sessions.read().await.get(id).map(|s| s.clone())
    }

    pub async fn update_session<F>(&self, id: &str, f: F) -> Result<(), String>
    where
        F: FnOnce(&mut Session),
    {
        if let Some(session) = self.sessions.write().await.get_mut(id) {
            f(session);
            session.update_activity();
            Ok(())
        } else {
            Err(format!("Session {} not found", id))
        }
    }

    pub async fn remove_session(&self, id: &str) -> Option<Session> {
        self.sessions.write().await.remove(id)
    }

    pub async fn list_sessions(&self) -> Vec<SessionInfo> {
        self.sessions
            .read()
            .await
            .iter()
            .map(|(_, session)| {
                SessionInfo {
                    id: session.id.clone(),
                    client_name: session.client_info.as_ref().map(|c| c.name.clone()),
                    created_at: session.created_at,
                    last_activity: session.last_activity,
                    context_files: session.context_files.len(),
                }
            })
            .collect()
    }

    pub async fn broadcast_to_all(&self, message: McpMessage) {
        for (_, session) in self.sessions.read().await.iter() {
            let _ = session.send_message(message.clone());
        }
    }

    pub async fn cleanup_inactive(&self, timeout_minutes: i64) {
        let cutoff = Utc::now() - chrono::Duration::minutes(timeout_minutes);
        let inactive: Vec<_> = self.sessions
            .read()
            .await
            .iter()
            .filter(|(_, s)| s.last_activity < cutoff)
            .map(|(_, s)| s.id.clone())
            .collect();
        
        for id in inactive {
            self.remove_session(&id).await;
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionInfo {
    pub id: String,
    pub client_name: Option<String>,
    pub created_at: DateTime<Utc>,
    pub last_activity: DateTime<Utc>,
    pub context_files: usize,
}

impl Default for SessionManager {
    fn default() -> Self {
        Self::new()
    }
}