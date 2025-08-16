//! Conversational request handler for IDE interactions

use crate::chat::{ChatInterface, ChatMessage, CodeBlock};
use crate::ide::CodeEditor;
use crate::element::ElementGraph;
use crate::error::UiResult;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

/// Request types that can be made through conversation
#[derive(Debug, Clone)]
pub enum ConversationRequest {
    ShowCode {
        file_path: String,
        function_name: Option<String>,
        line_range: Option<std::ops::Range<usize>>,
    },
    SearchCode {
        query: String,
        file_pattern: Option<String>,
    },
    RunCommand {
        command: String,
    },
    SaveCode {
        file_path: String,
        content: String,
    },
    ExplainCode {
        code: String,
        language: String,
    },
    RefactorCode {
        original: String,
        instructions: String,
    },
}

/// Context for finding code elements
pub struct CodeContext {
    /// Map of file paths to their content
    files: HashMap<String, String>,
    /// Currently open files in editors
    open_editors: Vec<Arc<RwLock<CodeEditor>>>,
}

impl CodeContext {
    pub fn new() -> Self {
        Self {
            files: HashMap::new(),
            open_editors: Vec::new(),
        }
    }
    
    /// Find a function by name
    pub fn find_function(&self, name: &str) -> Option<(String, std::ops::Range<usize>)> {
        for (path, content) in &self.files {
            if let Some(range) = self.find_function_in_file(content, name) {
                return Some((path.clone(), range));
            }
        }
        None
    }
    
    /// Find a function in a specific file
    fn find_function_in_file(&self, content: &str, name: &str) -> Option<std::ops::Range<usize>> {
        let lines: Vec<&str> = content.lines().collect();
        let mut in_function = false;
        let mut start_line = 0;
        let mut brace_count = 0;
        
        for (idx, line) in lines.iter().enumerate() {
            // Simple heuristic for Rust functions
            if line.contains(&format!("fn {}", name)) {
                in_function = true;
                start_line = idx;
                brace_count = 0;
            }
            
            if in_function {
                brace_count += line.matches('{').count() as i32;
                brace_count -= line.matches('}').count() as i32;
                
                if brace_count == 0 && line.contains('}') {
                    return Some(start_line..idx + 1);
                }
            }
        }
        
        None
    }
    
    /// Search for code matching a pattern
    pub fn search_code(&self, query: &str, file_pattern: Option<&str>) -> Vec<(String, usize, String)> {
        let mut results = Vec::new();
        
        for (path, content) in &self.files {
            // Filter by file pattern if provided
            if let Some(pattern) = file_pattern {
                if !path.contains(pattern) {
                    continue;
                }
            }
            
            // Search for query in file
            for (line_num, line) in content.lines().enumerate() {
                if line.contains(query) {
                    results.push((path.clone(), line_num, line.to_string()));
                }
            }
        }
        
        results
    }
}

/// Handles conversational requests and coordinates with UI elements
pub struct ConversationHandler {
    chat_interface: Arc<RwLock<ChatInterface>>,
    code_context: Arc<RwLock<CodeContext>>,
    element_graph: Arc<RwLock<ElementGraph>>,
}

impl ConversationHandler {
    pub fn new(
        chat_interface: Arc<RwLock<ChatInterface>>,
        element_graph: Arc<RwLock<ElementGraph>>,
    ) -> Self {
        Self {
            chat_interface,
            code_context: Arc::new(RwLock::new(CodeContext::new())),
            element_graph,
        }
    }
    
    /// Process a conversational request
    pub async fn handle_request(&self, request: ConversationRequest) -> UiResult<()> {
        match request {
            ConversationRequest::ShowCode { file_path, function_name, line_range } => {
                self.show_code(file_path, function_name, line_range).await?;
            }
            ConversationRequest::SearchCode { query, file_pattern } => {
                self.search_code(query, file_pattern).await?;
            }
            ConversationRequest::RunCommand { command } => {
                self.run_command(command).await?;
            }
            ConversationRequest::SaveCode { file_path, content } => {
                self.save_code(file_path, content).await?;
            }
            ConversationRequest::ExplainCode { code, language } => {
                self.explain_code(code, language).await?;
            }
            ConversationRequest::RefactorCode { original, instructions } => {
                self.refactor_code(original, instructions).await?;
            }
        }
        Ok(())
    }
    
    /// Show code in the chat interface with inline editing
    async fn show_code(
        &self,
        file_path: String,
        function_name: Option<String>,
        line_range: Option<std::ops::Range<usize>>,
    ) -> UiResult<()> {
        let context = self.code_context.read().await;
        
        // Load file content
        let content = if let Some(file_content) = context.files.get(&file_path) {
            file_content.clone()
        } else {
            // TODO: Load from filesystem
            String::new()
        };
        
        // Find specific function or use line range
        let (code_snippet, focused_lines) = if let Some(func_name) = function_name {
            if let Some((_, range)) = context.find_function(&func_name) {
                let lines: Vec<&str> = content.lines().collect();
                let snippet = lines[range.clone()].join("\n");
                (snippet, Some(range))
            } else {
                (content.clone(), None)
            }
        } else if let Some(range) = line_range {
            let lines: Vec<&str> = content.lines().collect();
            let snippet = lines[range.clone()].join("\n");
            (snippet, Some(range))
        } else {
            (content.clone(), None)
        };
        
        // Create code block
        let code_block = CodeBlock {
            id: Uuid::new_v4(),
            language: self.detect_language(&file_path),
            content: code_snippet,
            editable: true,
            focused_lines,
        };
        
        // Add to chat
        let mut chat = self.chat_interface.write().await;
        chat.add_assistant_message(
            format!("Here's the code from {}:", file_path),
            vec![code_block],
        );
        
        Ok(())
    }
    
    /// Search for code and display results
    async fn search_code(
        &self,
        query: String,
        file_pattern: Option<String>,
    ) -> UiResult<()> {
        let context = self.code_context.read().await;
        let results = context.search_code(&query, file_pattern.as_deref());
        
        let mut code_blocks = Vec::new();
        for (path, line_num, line) in results.iter().take(5) {
            code_blocks.push(CodeBlock {
                id: Uuid::new_v4(),
                language: self.detect_language(path),
                content: format!("{}:{}: {}", path, line_num + 1, line),
                editable: false,
                focused_lines: None,
            });
        }
        
        let message = if results.is_empty() {
            format!("No results found for '{}'", query)
        } else {
            format!("Found {} results for '{}':", results.len(), query)
        };
        
        let mut chat = self.chat_interface.write().await;
        chat.add_assistant_message(message, code_blocks);
        
        Ok(())
    }
    
    /// Run a command and show output
    async fn run_command(&self, command: String) -> UiResult<()> {
        // TODO: Execute command through terminal
        let output = format!("$ {}\n[Command execution not yet implemented]", command);
        
        let code_block = CodeBlock {
            id: Uuid::new_v4(),
            language: "bash".to_string(),
            content: output,
            editable: false,
            focused_lines: None,
        };
        
        let mut chat = self.chat_interface.write().await;
        chat.add_assistant_message(
            "Command output:".to_string(),
            vec![code_block],
        );
        
        Ok(())
    }
    
    /// Save code to a file
    async fn save_code(&self, file_path: String, content: String) -> UiResult<()> {
        // TODO: Save through server API
        let mut context = self.code_context.write().await;
        context.files.insert(file_path.clone(), content);
        
        let mut chat = self.chat_interface.write().await;
        chat.add_assistant_message(
            format!("Code saved to {}", file_path),
            Vec::new(),
        );
        
        Ok(())
    }
    
    /// Explain code functionality
    async fn explain_code(&self, code: String, language: String) -> UiResult<()> {
        // TODO: Integrate with AI service for explanations
        let explanation = "Code explanation would be generated here using AI analysis.";
        
        let mut chat = self.chat_interface.write().await;
        chat.add_assistant_message(
            explanation.to_string(),
            vec![CodeBlock {
                id: Uuid::new_v4(),
                language,
                content: code,
                editable: false,
                focused_lines: None,
            }],
        );
        
        Ok(())
    }
    
    /// Refactor code based on instructions
    async fn refactor_code(&self, original: String, instructions: String) -> UiResult<()> {
        // TODO: Integrate with AI service for refactoring
        let refactored = original.clone(); // Placeholder
        
        let mut chat = self.chat_interface.write().await;
        chat.add_assistant_message(
            format!("Refactored code based on: {}", instructions),
            vec![CodeBlock {
                id: Uuid::new_v4(),
                language: "rust".to_string(),
                content: refactored,
                editable: true,
                focused_lines: None,
            }],
        );
        
        Ok(())
    }
    
    /// Detect language from file extension
    fn detect_language(&self, file_path: &str) -> String {
        if file_path.ends_with(".rs") {
            "rust".to_string()
        } else if file_path.ends_with(".js") || file_path.ends_with(".jsx") {
            "javascript".to_string()
        } else if file_path.ends_with(".ts") || file_path.ends_with(".tsx") {
            "typescript".to_string()
        } else if file_path.ends_with(".py") {
            "python".to_string()
        } else if file_path.ends_with(".go") {
            "go".to_string()
        } else if file_path.ends_with(".cpp") || file_path.ends_with(".cc") {
            "cpp".to_string()
        } else if file_path.ends_with(".c") {
            "c".to_string()
        } else if file_path.ends_with(".java") {
            "java".to_string()
        } else if file_path.ends_with(".sh") || file_path.ends_with(".bash") {
            "bash".to_string()
        } else {
            "plain".to_string()
        }
    }
    
    /// Parse natural language message into conversation request
    pub fn parse_message(&self, message: &str) -> Option<ConversationRequest> {
        let lower = message.to_lowercase();
        
        // Pattern matching for common requests
        if lower.contains("show me") || lower.contains("display") {
            if lower.contains("function") {
                // Extract function name
                let words: Vec<&str> = message.split_whitespace().collect();
                if let Some(idx) = words.iter().position(|&w| w == "function") {
                    if idx + 1 < words.len() {
                        let func_name = words[idx + 1].trim_matches(|c: char| !c.is_alphanumeric());
                        return Some(ConversationRequest::ShowCode {
                            file_path: String::new(),
                            function_name: Some(func_name.to_string()),
                            line_range: None,
                        });
                    }
                }
            }
        } else if lower.contains("search for") || lower.contains("find") {
            // Extract search query
            if let Some(start) = lower.find("search for") {
                let query = &message[start + 10..].trim();
                return Some(ConversationRequest::SearchCode {
                    query: query.to_string(),
                    file_pattern: None,
                });
            } else if let Some(start) = lower.find("find") {
                let query = &message[start + 4..].trim();
                return Some(ConversationRequest::SearchCode {
                    query: query.to_string(),
                    file_pattern: None,
                });
            }
        } else if lower.contains("run") || lower.contains("execute") {
            // Extract command
            if let Some(start) = message.find('`') {
                if let Some(end) = message[start + 1..].find('`') {
                    let command = &message[start + 1..start + 1 + end];
                    return Some(ConversationRequest::RunCommand {
                        command: command.to_string(),
                    });
                }
            }
        } else if lower.contains("explain") {
            // Code explanation request
            return Some(ConversationRequest::ExplainCode {
                code: String::new(),
                language: "rust".to_string(),
            });
        }
        
        None
    }
}