use crate::components::*;
use crate::channel_manager::ChannelManager;
use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;

/// Handles message creation, formatting, and bubble state management
pub struct MessageSystem {
    channel_manager: Arc<ChannelManager>,
}

impl MessageSystem {
    pub fn new(channel_manager: Arc<ChannelManager>) -> Self {
        Self { channel_manager }
    }

    // ========================================================================
    // Message Creation
    // ========================================================================

    pub async fn send_text_message(
        &self,
        channel_id: Uuid,
        sender: AgentId,
        text: String,
    ) -> Result<Uuid> {
        let content = MessageContent::Text(text);
        self.channel_manager
            .send_message(channel_id, sender, content)
            .await
    }

    pub async fn send_code_message(
        &self,
        channel_id: Uuid,
        sender: AgentId,
        language: String,
        code: String,
    ) -> Result<Uuid> {
        let content = MessageContent::Code {
            language,
            content: code,
        };
        self.channel_manager
            .send_message(channel_id, sender, content)
            .await
    }

    pub async fn send_inline_editor(
        &self,
        channel_id: Uuid,
        sender: AgentId,
        file_path: String,
        content: String,
        language: String,
    ) -> Result<Uuid> {
        let editor = InlineEditor {
            message_id: Uuid::new_v4(), // Will be updated after message creation
            file_path,
            content: content.clone(),
            language,
            vim_mode: VimMode::Normal,
            cursor_position: (0, 0),
            selection: None,
            is_expanded: true,
            is_dirty: false,
            original_content: Some(content),
        };

        let message_content = MessageContent::InlineEditor(editor);
        let message_id = self
            .channel_manager
            .send_message(channel_id, sender, message_content)
            .await?;

        // Update the editor's message_id
        self.update_inline_component_id(channel_id, message_id, message_id)
            .await?;

        Ok(message_id)
    }

    pub async fn send_inline_file_browser(
        &self,
        channel_id: Uuid,
        sender: AgentId,
        path: std::path::PathBuf,
        entries: Vec<FileEntry>,
    ) -> Result<Uuid> {
        let browser = InlineFileBrowser {
            message_id: Uuid::new_v4(), // Will be updated after message creation
            current_path: path,
            entries,
            expanded_dirs: Vec::new(),
            selected_entry: None,
            is_expanded: true,
        };

        let message_content = MessageContent::InlineFileBrowser(browser);
        let message_id = self
            .channel_manager
            .send_message(channel_id, sender, message_content)
            .await?;

        // Update the browser's message_id
        self.update_inline_component_id(channel_id, message_id, message_id)
            .await?;

        Ok(message_id)
    }

    pub async fn send_inline_terminal(
        &self,
        channel_id: Uuid,
        sender: AgentId,
        session_id: String,
        initial_output: Vec<String>,
    ) -> Result<Uuid> {
        let terminal = InlineTerminal {
            message_id: Uuid::new_v4(), // Will be updated after message creation
            session_id,
            output_buffer: initial_output,
            current_directory: std::env::current_dir().unwrap_or_default(),
            is_expanded: true,
            max_lines: 1000,
        };

        let message_content = MessageContent::InlineTerminal(terminal);
        let message_id = self
            .channel_manager
            .send_message(channel_id, sender, message_content)
            .await?;

        // Update the terminal's message_id
        self.update_inline_component_id(channel_id, message_id, message_id)
            .await?;

        Ok(message_id)
    }

    pub async fn send_inline_diff(
        &self,
        channel_id: Uuid,
        sender: AgentId,
        file_path: String,
        hunks: Vec<DiffHunk>,
    ) -> Result<Uuid> {
        let diff = InlineDiff {
            message_id: Uuid::new_v4(), // Will be updated after message creation
            file_path,
            hunks,
            is_expanded: true,
        };

        let message_content = MessageContent::InlineDiff(diff);
        let message_id = self
            .channel_manager
            .send_message(channel_id, sender, message_content)
            .await?;

        // Update the diff's message_id
        self.update_inline_component_id(channel_id, message_id, message_id)
            .await?;

        Ok(message_id)
    }

    pub async fn send_system_notification(
        &self,
        channel_id: Uuid,
        notification: String,
    ) -> Result<Uuid> {
        let content = MessageContent::SystemNotification(notification);
        // System notifications use a special system agent ID
        let system_agent = AgentId(Uuid::nil());
        self.channel_manager
            .send_message(channel_id, system_agent, content)
            .await
    }

    // ========================================================================
    // Inline Component Updates
    // ========================================================================

    async fn update_inline_component_id(
        &self,
        channel_id: Uuid,
        message_id: Uuid,
        new_id: Uuid,
    ) -> Result<()> {
        // This is a helper to update the message_id field in inline components
        // after the message has been created
        // In a real implementation, this would update the component in the ECS
        Ok(())
    }

    pub async fn update_editor_content(
        &self,
        channel_id: Uuid,
        message_id: Uuid,
        new_content: String,
    ) -> Result<()> {
        // Update the content of an inline editor
        // This would modify the MessageContent::InlineEditor variant
        tracing::debug!("Updating editor content for message {}", message_id);
        Ok(())
    }

    pub async fn update_terminal_output(
        &self,
        channel_id: Uuid,
        message_id: Uuid,
        new_lines: Vec<String>,
    ) -> Result<()> {
        // Append new output to a terminal
        tracing::debug!("Appending {} lines to terminal {}", new_lines.len(), message_id);
        Ok(())
    }

    pub async fn update_file_browser_path(
        &self,
        channel_id: Uuid,
        message_id: Uuid,
        new_path: std::path::PathBuf,
        entries: Vec<FileEntry>,
    ) -> Result<()> {
        // Update the current path and entries of a file browser
        tracing::debug!("Updating file browser path to {:?}", new_path);
        Ok(())
    }

    // ========================================================================
    // Bubble State Management
    // ========================================================================

    pub async fn set_all_bubbles_state(
        &self,
        channel_id: Uuid,
        state: BubbleState,
    ) -> Result<()> {
        let messages = self.channel_manager.get_messages(channel_id).await?;
        for message in messages {
            self.channel_manager
                .update_bubble_state(channel_id, message.id, state.clone())
                .await?;
        }
        Ok(())
    }

    pub async fn collapse_all_bubbles(&self, channel_id: Uuid) -> Result<()> {
        self.set_all_bubbles_state(channel_id, BubbleState::Collapsed)
            .await
    }

    pub async fn expand_all_bubbles(&self, channel_id: Uuid) -> Result<()> {
        self.set_all_bubbles_state(channel_id, BubbleState::Expanded)
            .await
    }

    pub async fn compress_all_bubbles(&self, channel_id: Uuid) -> Result<()> {
        self.set_all_bubbles_state(channel_id, BubbleState::Compressed)
            .await
    }

    // ========================================================================
    // Message Formatting
    // ========================================================================

    pub fn format_message_preview(&self, message: &MessageComponent) -> String {
        match &message.content {
            MessageContent::Text(text) => {
                // Return first 100 chars for preview
                if text.len() > 100 {
                    format!("{}...", &text[..100])
                } else {
                    text.clone()
                }
            }
            MessageContent::Code { language, content } => {
                format!("[Code: {}] {} lines", language, content.lines().count())
            }
            MessageContent::InlineEditor(editor) => {
                format!("[Editor: {}]", editor.file_path)
            }
            MessageContent::InlineFileBrowser(browser) => {
                format!("[Files: {:?}]", browser.current_path)
            }
            MessageContent::InlineTerminal(terminal) => {
                format!("[Terminal: {}]", terminal.session_id)
            }
            MessageContent::InlineDiff(diff) => {
                format!("[Diff: {}]", diff.file_path)
            }
            MessageContent::SystemNotification(text) => {
                format!("[System: {}]", text)
            }
        }
    }

    pub fn format_message_for_bubble(&self, message: &MessageComponent) -> BubbleContent {
        let preview = self.format_message_preview(message);
        
        match message.bubble_state {
            BubbleState::Collapsed => BubbleContent {
                title: preview,
                body: None,
                expandable: true,
            },
            BubbleState::Compressed => {
                let body = match &message.content {
                    MessageContent::Text(text) => Some(text[..text.len().min(500)].to_string()),
                    MessageContent::Code { content, .. } => {
                        // Show first 10 lines
                        Some(
                            content
                                .lines()
                                .take(10)
                                .collect::<Vec<_>>()
                                .join("\n")
                        )
                    }
                    MessageContent::InlineEditor(editor) => {
                        // Show first 10 lines of editor content
                        Some(
                            editor
                                .content
                                .lines()
                                .take(10)
                                .collect::<Vec<_>>()
                                .join("\n")
                        )
                    }
                    _ => None,
                };
                
                BubbleContent {
                    title: preview,
                    body,
                    expandable: true,
                }
            }
            BubbleState::Expanded => BubbleContent {
                title: preview,
                body: Some(self.render_full_content(&message.content)),
                expandable: true,
            },
        }
    }

    fn render_full_content(&self, content: &MessageContent) -> String {
        match content {
            MessageContent::Text(text) => text.clone(),
            MessageContent::Code { language, content } => {
                format!("```{}\n{}\n```", language, content)
            }
            MessageContent::InlineEditor(editor) => {
                format!(
                    "File: {}\nMode: {:?}\n```{}\n{}\n```",
                    editor.file_path, editor.vim_mode, editor.language, editor.content
                )
            }
            MessageContent::InlineFileBrowser(browser) => {
                let entries_str = browser
                    .entries
                    .iter()
                    .map(|e| {
                        format!(
                            "{} {}",
                            if e.is_directory { "📁" } else { "📄" },
                            e.name
                        )
                    })
                    .collect::<Vec<_>>()
                    .join("\n");
                format!("Path: {:?}\n{}", browser.current_path, entries_str)
            }
            MessageContent::InlineTerminal(terminal) => {
                format!(
                    "Session: {}\nDirectory: {:?}\n{}",
                    terminal.session_id,
                    terminal.current_directory,
                    terminal.output_buffer.join("\n")
                )
            }
            MessageContent::InlineDiff(diff) => {
                let hunks_str = diff
                    .hunks
                    .iter()
                    .map(|hunk| self.format_diff_hunk(hunk))
                    .collect::<Vec<_>>()
                    .join("\n---\n");
                format!("Diff: {}\n{}", diff.file_path, hunks_str)
            }
            MessageContent::SystemNotification(text) => format!("⚠️ {}", text),
        }
    }

    fn format_diff_hunk(&self, hunk: &DiffHunk) -> String {
        let header = format!(
            "@@ -{},{} +{},{} @@",
            hunk.old_start, hunk.old_lines, hunk.new_start, hunk.new_lines
        );
        
        let lines_str = hunk
            .lines
            .iter()
            .map(|line| match line {
                DiffLine::Context(s) => format!(" {}", s),
                DiffLine::Added(s) => format!("+{}", s),
                DiffLine::Removed(s) => format!("-{}", s),
            })
            .collect::<Vec<_>>()
            .join("\n");
        
        format!("{}\n{}", header, lines_str)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BubbleContent {
    pub title: String,
    pub body: Option<String>,
    pub expandable: bool,
}