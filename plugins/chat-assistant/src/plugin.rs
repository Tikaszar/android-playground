use playground_plugin::Plugin;
use playground_types::{
    PluginMetadata, PluginId, Version, Event,
    context::Context,
    render_context::RenderContext,
    error::PluginError,
};
use tracing::{info, debug};

use crate::chat_view::{ChatView, MessageSender};

pub struct ChatAssistantPlugin {
    metadata: PluginMetadata,
    chat_view: Option<ChatView>,
    channel_id: Option<u16>,
}

impl ChatAssistantPlugin {
    pub fn new() -> Self {
        Self {
            metadata: PluginMetadata {
                id: PluginId("chat-assistant".to_string()),
                name: "Chat Assistant".to_string(),
                version: Version {
                    major: 0,
                    minor: 1,
                    patch: 0,
                },
            },
            chat_view: None,
            channel_id: None,
        }
    }

    fn initialize_chat(&mut self) {
        let mut chat = ChatView::new();
        
        // Add welcome message
        chat.add_message(
            MessageSender::Assistant,
            "Welcome to the Conversational IDE! I'm here to help you with your code. You can:\n\
            • Ask me to explain code\n\
            • Request code generation\n\
            • Get help with debugging\n\
            • Learn about best practices\n\n\
            Type your message below and press Enter to send.".to_string()
        );
        
        self.chat_view = Some(chat);
    }

    pub fn process_user_message(&mut self, message: String) {
        // Generate response before mutating chat
        let response = self.generate_response(&message);
        
        if let Some(chat) = &mut self.chat_view {
            // Add user message
            chat.add_message(MessageSender::User, message);
            
            // Add assistant response
            chat.add_message(MessageSender::Assistant, response);
        }
    }

    fn generate_response(&self, message: &str) -> String {
        // This is a placeholder for actual AI integration
        // In a real implementation, this would call an AI API
        
        if message.to_lowercase().contains("hello") {
            "Hello! How can I assist you with your code today?".to_string()
        } else if message.to_lowercase().contains("help") {
            "I can help you with:\n\
            • Writing new code\n\
            • Understanding existing code\n\
            • Debugging issues\n\
            • Refactoring\n\
            • Testing strategies\n\n\
            What would you like help with?".to_string()
        } else if message.to_lowercase().contains("code") {
            "I'd be happy to help with code! Here's an example:\n\n\
            ```rust\n\
            fn example_function() {\n    \
                println!(\"This is example code\");\n\
            }\n\
            ```\n\n\
            What specific code would you like me to help with?".to_string()
        } else {
            format!("I understand you're asking about: {}\n\
                    Let me help you with that...", message)
        }
    }
}

impl Plugin for ChatAssistantPlugin {
    fn metadata(&self) -> &PluginMetadata {
        &self.metadata
    }

    fn on_load(&mut self, _ctx: &mut Context) -> Result<(), PluginError> {
        info!("Chat assistant plugin loading");
        
        // Register with networking system for channels 1050-1059
        self.channel_id = Some(1050);
        
        // Initialize chat interface
        self.initialize_chat();
        
        info!("Chat assistant plugin loaded successfully");
        Ok(())
    }

    fn on_unload(&mut self, _ctx: &mut Context) {
        info!("Chat assistant plugin unloading");
        
        // Clean up resources
        self.chat_view = None;
    }

    fn update(&mut self, _ctx: &mut Context, _delta_time: f32) {
        // Update chat view if needed
        // This is where we'd handle animations, etc.
    }

    fn render(&mut self, _ctx: &mut RenderContext) {
        // Chat rendering is handled by the UI system
        // through the Element trait implementation
    }

    fn on_event(&mut self, event: &Event) -> bool {
        // Handle plugin events
        debug!("Chat assistant received event: {:?}", event);
        
        // Return true if event was handled
        false
    }
}

pub fn create() -> ChatAssistantPlugin {
    ChatAssistantPlugin::new()
}