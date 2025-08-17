use chrono::{DateTime, Utc};
use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::mpsc;
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
    sessions: Arc<DashMap<SessionId, Session>>,
}

impl SessionManager {
    pub fn new() -> Self {
        Self {
            sessions: Arc::new(DashMap::new()),
        }
    }

    pub fn create_session(&self, id: Option<String>) -> (SessionId, mpsc::UnboundedReceiver<McpMessage>) {
        let (session, rx) = Session::new(id);
        let session_id = session.id.clone();
        self.sessions.insert(session_id.clone(), session);
        (session_id, rx)
    }

    pub fn get_session(&self, id: &str) -> Option<Session> {
        self.sessions.get(id).map(|s| s.clone())
    }

    pub fn update_session<F>(&self, id: &str, f: F) -> Result<(), String>
    where
        F: FnOnce(&mut Session),
    {
        if let Some(mut session) = self.sessions.get_mut(id) {
            f(&mut session);
            session.update_activity();
            Ok(())
        } else {
            Err(format!("Session {} not found", id))
        }
    }

    pub fn remove_session(&self, id: &str) -> Option<Session> {
        self.sessions.remove(id).map(|(_, s)| s)
    }

    pub fn list_sessions(&self) -> Vec<SessionInfo> {
        self.sessions
            .iter()
            .map(|entry| {
                let session = entry.value();
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

    pub fn broadcast_to_all(&self, message: McpMessage) {
        for session in self.sessions.iter() {
            let _ = session.send_message(message.clone());
        }
    }

    pub fn cleanup_inactive(&self, timeout_minutes: i64) {
        let cutoff = Utc::now() - chrono::Duration::minutes(timeout_minutes);
        let inactive: Vec<_> = self.sessions
            .iter()
            .filter(|s| s.last_activity < cutoff)
            .map(|s| s.id.clone())
            .collect();
        
        for id in inactive {
            self.remove_session(&id);
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