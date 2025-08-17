use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::fs;
use tokio::process::Command;

use crate::mcp::error::{McpError, McpResult};
use crate::mcp::protocol::ToolDescription;

/// Tool trait that any LLM can execute
#[async_trait]
pub trait Tool: Send + Sync {
    fn name(&self) -> &str;
    fn description(&self) -> &str;
    fn input_schema(&self) -> Value;
    async fn execute(&self, arguments: Value) -> McpResult<Value>;
}

/// Provides tools to LLM clients
pub struct ToolProvider {
    tools: HashMap<String, Arc<dyn Tool>>,
}

impl ToolProvider {
    pub fn new() -> Self {
        let mut provider = Self {
            tools: HashMap::new(),
        };
        
        // Register all built-in tools
        provider.register(Arc::new(ReadFileTool));
        provider.register(Arc::new(WriteFileTool));
        provider.register(Arc::new(ListFilesTool));
        provider.register(Arc::new(SearchFilesTool));
        provider.register(Arc::new(ExecuteCommandTool));
        provider.register(Arc::new(CreateDirectoryTool));
        provider.register(Arc::new(DeleteFileTool));
        provider.register(Arc::new(MoveFileTool));
        
        provider
    }

    pub fn register(&mut self, tool: Arc<dyn Tool>) {
        self.tools.insert(tool.name().to_string(), tool);
    }

    pub fn get_tool(&self, name: &str) -> Option<Arc<dyn Tool>> {
        self.tools.get(name).cloned()
    }

    pub fn list_tools(&self) -> Vec<ToolDescription> {
        self.tools
            .values()
            .map(|tool| ToolDescription {
                name: tool.name().to_string(),
                description: tool.description().to_string(),
                input_schema: tool.input_schema(),
            })
            .collect()
    }
}

// --- Built-in Tools ---

/// Read file contents
struct ReadFileTool;

#[async_trait]
impl Tool for ReadFileTool {
    fn name(&self) -> &str {
        "read_file"
    }

    fn description(&self) -> &str {
        "Read the contents of a file"
    }

    fn input_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "path": {
                    "type": "string",
                    "description": "Path to the file to read"
                }
            },
            "required": ["path"]
        })
    }

    async fn execute(&self, arguments: Value) -> McpResult<Value> {
        let path = arguments["path"]
            .as_str()
            .ok_or_else(|| McpError::InvalidParameters("path must be a string".into()))?;
        
        let content = fs::read_to_string(path).await.map_err(|e| {
            McpError::ToolExecutionFailed(format!("Failed to read file: {}", e))
        })?;
        
        Ok(json!({
            "content": content,
            "path": path
        }))
    }
}

/// Write file contents
struct WriteFileTool;

#[async_trait]
impl Tool for WriteFileTool {
    fn name(&self) -> &str {
        "write_file"
    }

    fn description(&self) -> &str {
        "Write content to a file (creates if doesn't exist)"
    }

    fn input_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "path": {
                    "type": "string",
                    "description": "Path to the file to write"
                },
                "content": {
                    "type": "string",
                    "description": "Content to write to the file"
                }
            },
            "required": ["path", "content"]
        })
    }

    async fn execute(&self, arguments: Value) -> McpResult<Value> {
        let path = arguments["path"]
            .as_str()
            .ok_or_else(|| McpError::InvalidParameters("path must be a string".into()))?;
        
        let content = arguments["content"]
            .as_str()
            .ok_or_else(|| McpError::InvalidParameters("content must be a string".into()))?;
        
        fs::write(path, content).await.map_err(|e| {
            McpError::ToolExecutionFailed(format!("Failed to write file: {}", e))
        })?;
        
        Ok(json!({
            "success": true,
            "path": path,
            "bytes_written": content.len()
        }))
    }
}

/// List files in directory
struct ListFilesTool;

#[async_trait]
impl Tool for ListFilesTool {
    fn name(&self) -> &str {
        "list_files"
    }

    fn description(&self) -> &str {
        "List files and directories in a given path"
    }

    fn input_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "path": {
                    "type": "string",
                    "description": "Path to list (defaults to current directory)"
                }
            }
        })
    }

    async fn execute(&self, arguments: Value) -> McpResult<Value> {
        let path = arguments["path"]
            .as_str()
            .unwrap_or(".");
        
        let mut entries = fs::read_dir(path).await.map_err(|e| {
            McpError::ToolExecutionFailed(format!("Failed to list directory: {}", e))
        })?;
        
        let mut files = Vec::new();
        let mut dirs = Vec::new();
        
        while let Some(entry) = entries.next_entry().await.map_err(|e| {
            McpError::ToolExecutionFailed(format!("Failed to read entry: {}", e))
        })? {
            let metadata = entry.metadata().await.map_err(|e| {
                McpError::ToolExecutionFailed(format!("Failed to get metadata: {}", e))
            })?;
            
            let name = entry.file_name().to_string_lossy().to_string();
            
            if metadata.is_dir() {
                dirs.push(name);
            } else {
                files.push(json!({
                    "name": name,
                    "size": metadata.len()
                }));
            }
        }
        
        Ok(json!({
            "path": path,
            "directories": dirs,
            "files": files
        }))
    }
}

/// Search for patterns in files
struct SearchFilesTool;

#[async_trait]
impl Tool for SearchFilesTool {
    fn name(&self) -> &str {
        "search_files"
    }

    fn description(&self) -> &str {
        "Search for a pattern in files using ripgrep"
    }

    fn input_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "pattern": {
                    "type": "string",
                    "description": "Pattern to search for"
                },
                "path": {
                    "type": "string",
                    "description": "Path to search in (defaults to current directory)"
                },
                "file_type": {
                    "type": "string",
                    "description": "File type filter (e.g., 'rust', 'js', 'py')"
                }
            },
            "required": ["pattern"]
        })
    }

    async fn execute(&self, arguments: Value) -> McpResult<Value> {
        let pattern = arguments["pattern"]
            .as_str()
            .ok_or_else(|| McpError::InvalidParameters("pattern must be a string".into()))?;
        
        let path = arguments["path"].as_str().unwrap_or(".");
        
        let mut cmd = Command::new("rg");
        cmd.arg("--json")
           .arg(pattern)
           .arg(path);
        
        if let Some(file_type) = arguments["file_type"].as_str() {
            cmd.arg("-t").arg(file_type);
        }
        
        let output = cmd.output().await.map_err(|e| {
            McpError::ToolExecutionFailed(format!("Failed to execute ripgrep: {}", e))
        })?;
        
        let stdout = String::from_utf8_lossy(&output.stdout);
        
        // Parse ripgrep JSON output
        let mut matches = Vec::new();
        for line in stdout.lines() {
            if let Ok(result) = serde_json::from_str::<Value>(line) {
                if result["type"] == "match" {
                    matches.push(json!({
                        "file": result["data"]["path"]["text"],
                        "line": result["data"]["line_number"],
                        "text": result["data"]["lines"]["text"],
                    }));
                }
            }
        }
        
        Ok(json!({
            "pattern": pattern,
            "matches": matches,
            "total": matches.len()
        }))
    }
}

/// Execute shell command
struct ExecuteCommandTool;

#[async_trait]
impl Tool for ExecuteCommandTool {
    fn name(&self) -> &str {
        "execute_command"
    }

    fn description(&self) -> &str {
        "Execute a shell command"
    }

    fn input_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "command": {
                    "type": "string",
                    "description": "Command to execute"
                },
                "working_dir": {
                    "type": "string",
                    "description": "Working directory for the command"
                }
            },
            "required": ["command"]
        })
    }

    async fn execute(&self, arguments: Value) -> McpResult<Value> {
        let command = arguments["command"]
            .as_str()
            .ok_or_else(|| McpError::InvalidParameters("command must be a string".into()))?;
        
        let mut cmd = Command::new("sh");
        cmd.arg("-c").arg(command);
        
        if let Some(dir) = arguments["working_dir"].as_str() {
            cmd.current_dir(dir);
        }
        
        let output = cmd.output().await.map_err(|e| {
            McpError::ToolExecutionFailed(format!("Failed to execute command: {}", e))
        })?;
        
        Ok(json!({
            "command": command,
            "exit_code": output.status.code(),
            "stdout": String::from_utf8_lossy(&output.stdout),
            "stderr": String::from_utf8_lossy(&output.stderr),
        }))
    }
}

/// Create directory
struct CreateDirectoryTool;

#[async_trait]
impl Tool for CreateDirectoryTool {
    fn name(&self) -> &str {
        "create_directory"
    }

    fn description(&self) -> &str {
        "Create a directory (including parent directories)"
    }

    fn input_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "path": {
                    "type": "string",
                    "description": "Path of directory to create"
                }
            },
            "required": ["path"]
        })
    }

    async fn execute(&self, arguments: Value) -> McpResult<Value> {
        let path = arguments["path"]
            .as_str()
            .ok_or_else(|| McpError::InvalidParameters("path must be a string".into()))?;
        
        fs::create_dir_all(path).await.map_err(|e| {
            McpError::ToolExecutionFailed(format!("Failed to create directory: {}", e))
        })?;
        
        Ok(json!({
            "success": true,
            "path": path
        }))
    }
}

/// Delete file or directory
struct DeleteFileTool;

#[async_trait]
impl Tool for DeleteFileTool {
    fn name(&self) -> &str {
        "delete_file"
    }

    fn description(&self) -> &str {
        "Delete a file or empty directory"
    }

    fn input_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "path": {
                    "type": "string",
                    "description": "Path to delete"
                }
            },
            "required": ["path"]
        })
    }

    async fn execute(&self, arguments: Value) -> McpResult<Value> {
        let path = arguments["path"]
            .as_str()
            .ok_or_else(|| McpError::InvalidParameters("path must be a string".into()))?;
        
        let metadata = fs::metadata(path).await.map_err(|e| {
            McpError::ToolExecutionFailed(format!("Failed to get metadata: {}", e))
        })?;
        
        if metadata.is_dir() {
            fs::remove_dir(path).await.map_err(|e| {
                McpError::ToolExecutionFailed(format!("Failed to delete directory: {}", e))
            })?;
        } else {
            fs::remove_file(path).await.map_err(|e| {
                McpError::ToolExecutionFailed(format!("Failed to delete file: {}", e))
            })?;
        }
        
        Ok(json!({
            "success": true,
            "path": path
        }))
    }
}

/// Move or rename file
struct MoveFileTool;

#[async_trait]
impl Tool for MoveFileTool {
    fn name(&self) -> &str {
        "move_file"
    }

    fn description(&self) -> &str {
        "Move or rename a file or directory"
    }

    fn input_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "from": {
                    "type": "string",
                    "description": "Source path"
                },
                "to": {
                    "type": "string",
                    "description": "Destination path"
                }
            },
            "required": ["from", "to"]
        })
    }

    async fn execute(&self, arguments: Value) -> McpResult<Value> {
        let from = arguments["from"]
            .as_str()
            .ok_or_else(|| McpError::InvalidParameters("from must be a string".into()))?;
        
        let to = arguments["to"]
            .as_str()
            .ok_or_else(|| McpError::InvalidParameters("to must be a string".into()))?;
        
        fs::rename(from, to).await.map_err(|e| {
            McpError::ToolExecutionFailed(format!("Failed to move file: {}", e))
        })?;
        
        Ok(json!({
            "success": true,
            "from": from,
            "to": to
        }))
    }
}

impl Default for ToolProvider {
    fn default() -> Self {
        Self::new()
    }
}