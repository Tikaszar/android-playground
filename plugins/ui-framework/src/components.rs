use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
use std::path::PathBuf;
use uuid::Uuid;

// ============================================================================
// Core Chat Components
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ChannelType {
    Direct,  // DM between two agents
    Group,   // Group chat with multiple agents
    System,  // System notifications channel
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChannelComponent {
    pub id: Uuid,
    pub name: String,
    pub channel_type: ChannelType,
    pub participants: Vec<AgentId>,
    pub created_at: DateTime<Utc>,
    pub last_message_at: Option<DateTime<Utc>>,
    pub unread_count: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BubbleState {
    Collapsed,   // Title + timestamp only
    Compressed,  // Relevant lines/content (MCP-specified)
    Expanded,    // Full content with scrolling
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MessageContent {
    Text(String),
    Code { language: String, content: String },
    InlineEditor(InlineEditor),
    InlineFileBrowser(InlineFileBrowser),
    InlineTerminal(InlineTerminal),
    InlineDiff(InlineDiff),
    SystemNotification(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageComponent {
    pub id: Uuid,
    pub channel_id: Uuid,
    pub sender: AgentId,
    pub content: MessageContent,
    pub timestamp: DateTime<Utc>,
    pub bubble_state: BubbleState,
    pub edited_at: Option<DateTime<Utc>>,
    pub reply_to: Option<Uuid>,
}

// ============================================================================
// Inline Components
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum VimMode {
    Normal,
    Insert,
    Visual,
    Command,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InlineEditor {
    pub message_id: Uuid,
    pub file_path: String,
    pub content: String,
    pub language: String,
    pub vim_mode: VimMode,
    pub cursor_position: (usize, usize), // (line, column)
    pub selection: Option<(usize, usize, usize, usize)>, // (start_line, start_col, end_line, end_col)
    pub is_expanded: bool,
    pub is_dirty: bool,
    pub original_content: Option<String>, // For showing diffs
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileEntry {
    pub name: String,
    pub path: PathBuf,
    pub is_directory: bool,
    pub size: Option<u64>,
    pub git_status: Option<GitStatus>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GitStatus {
    Modified,
    Added,
    Deleted,
    Renamed,
    Untracked,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InlineFileBrowser {
    pub message_id: Uuid,
    pub current_path: PathBuf,
    pub entries: Vec<FileEntry>,
    pub expanded_dirs: Vec<PathBuf>,
    pub selected_entry: Option<PathBuf>,
    pub is_expanded: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InlineTerminal {
    pub message_id: Uuid,
    pub session_id: String,
    pub output_buffer: Vec<String>,
    pub current_directory: PathBuf,
    pub is_expanded: bool,
    pub max_lines: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiffHunk {
    pub old_start: usize,
    pub old_lines: usize,
    pub new_start: usize,
    pub new_lines: usize,
    pub lines: Vec<DiffLine>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DiffLine {
    Context(String),
    Added(String),
    Removed(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InlineDiff {
    pub message_id: Uuid,
    pub file_path: String,
    pub hunks: Vec<DiffHunk>,
    pub is_expanded: bool,
}

// ============================================================================
// Agent Components
// ============================================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct AgentId(pub Uuid);

impl AgentId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AgentType {
    Orchestrator, // Manages tasks and assigns work
    Worker,       // Executes assigned tasks
    Human,        // Human user
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AgentStatus {
    Busy,    // Currently working on a task
    Idle,    // Available for new tasks
    Waiting, // Waiting for input or resources
    Offline, // Not connected
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentPermissions {
    pub can_execute_commands: bool,
    pub can_modify_files: bool,
    pub can_create_worktrees: bool,
    pub can_assign_tasks: bool,
}

impl Default for AgentPermissions {
    fn default() -> Self {
        Self {
            can_execute_commands: false,
            can_modify_files: false,
            can_create_worktrees: false,
            can_assign_tasks: false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentComponent {
    pub id: AgentId,
    pub name: String,
    pub agent_type: AgentType,
    pub status: AgentStatus,
    pub worktree_path: Option<PathBuf>,
    pub permissions: AgentPermissions,
    pub current_task: Option<TaskId>,
    pub last_active: DateTime<Utc>,
}

// ============================================================================
// Task Queue Components
// ============================================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TaskId(pub Uuid);

impl TaskId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TaskPriority {
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TaskStatus {
    Pending,
    Assigned(AgentId),
    InProgress(AgentId),
    Completed,
    Failed(String),
    Cancelled,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    pub id: TaskId,
    pub title: String,
    pub description: String,
    pub priority: TaskPriority,
    pub status: TaskStatus,
    pub created_at: DateTime<Utc>,
    pub assigned_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
    pub dependencies: Vec<TaskId>,
    pub context_files: Vec<PathBuf>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskResult {
    pub task_id: TaskId,
    pub success: bool,
    pub output: String,
    pub files_modified: Vec<PathBuf>,
    pub duration_seconds: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskQueueComponent {
    pub pending: VecDeque<Task>,
    pub active: HashMap<AgentId, Task>,
    pub completed: Vec<TaskResult>,
    pub task_history: HashMap<TaskId, Task>,
}

impl TaskQueueComponent {
    pub fn new() -> Self {
        Self {
            pending: VecDeque::new(),
            active: HashMap::new(),
            completed: Vec::new(),
            task_history: HashMap::new(),
        }
    }

    pub fn add_task(&mut self, task: Task) {
        let task_id = task.id;
        self.task_history.insert(task_id, task.clone());
        self.pending.push_back(task);
    }

    pub fn assign_task(&mut self, agent_id: AgentId) -> Option<Task> {
        if self.active.contains_key(&agent_id) {
            return None; // Agent already has a task
        }

        // Find next available task
        if let Some(mut task) = self.pending.pop_front() {
            task.status = TaskStatus::Assigned(agent_id);
            task.assigned_at = Some(Utc::now());
            self.active.insert(agent_id, task.clone());
            self.task_history.insert(task.id, task.clone());
            Some(task)
        } else {
            None
        }
    }

    pub fn complete_task(&mut self, agent_id: AgentId, result: TaskResult) {
        if let Some(mut task) = self.active.remove(&agent_id) {
            task.status = if result.success {
                TaskStatus::Completed
            } else {
                TaskStatus::Failed(result.output.clone())
            };
            task.completed_at = Some(Utc::now());
            self.task_history.insert(task.id, task);
            self.completed.push(result);
        }
    }

    pub fn get_agent_task(&self, agent_id: &AgentId) -> Option<&Task> {
        self.active.get(agent_id)
    }
    
    pub fn get_in_progress_tasks(&self) -> Vec<&Task> {
        self.active.values().collect()
    }
}

// ============================================================================
// UI State Components
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConversationState {
    pub active_channel: Option<Uuid>,
    pub channels: HashMap<Uuid, ChannelComponent>,
    pub messages: HashMap<Uuid, Vec<MessageComponent>>,
    pub typing_indicators: HashMap<AgentId, Uuid>, // agent -> channel they're typing in
}

impl ConversationState {
    pub fn new() -> Self {
        Self {
            active_channel: None,
            channels: HashMap::new(),
            messages: HashMap::new(),
            typing_indicators: HashMap::new(),
        }
    }

    pub fn create_channel(&mut self, name: String, channel_type: ChannelType, participants: Vec<AgentId>) -> Uuid {
        let channel = ChannelComponent {
            id: Uuid::new_v4(),
            name,
            channel_type,
            participants,
            created_at: Utc::now(),
            last_message_at: None,
            unread_count: 0,
        };
        let channel_id = channel.id;
        self.channels.insert(channel_id, channel);
        self.messages.insert(channel_id, Vec::new());
        channel_id
    }

    pub fn add_message(&mut self, channel_id: Uuid, sender: AgentId, content: MessageContent) -> Option<Uuid> {
        if let Some(messages) = self.messages.get_mut(&channel_id) {
            let message = MessageComponent {
                id: Uuid::new_v4(),
                channel_id,
                sender,
                content,
                timestamp: Utc::now(),
                bubble_state: BubbleState::Expanded,
                edited_at: None,
                reply_to: None,
            };
            let message_id = message.id;
            messages.push(message);
            
            // Update channel's last message time
            if let Some(channel) = self.channels.get_mut(&channel_id) {
                channel.last_message_at = Some(Utc::now());
            }
            
            Some(message_id)
        } else {
            None
        }
    }

    pub fn toggle_bubble_state(&mut self, channel_id: Uuid, message_id: Uuid) {
        if let Some(messages) = self.messages.get_mut(&channel_id) {
            if let Some(message) = messages.iter_mut().find(|m| m.id == message_id) {
                message.bubble_state = match message.bubble_state {
                    BubbleState::Collapsed => BubbleState::Compressed,
                    BubbleState::Compressed => BubbleState::Expanded,
                    BubbleState::Expanded => BubbleState::Collapsed,
                };
            }
        }
    }
}