use std::sync::Arc;
use std::collections::HashMap;
use axum::{
    Router,
    routing::{get, post},
    extract::State,
    response::{IntoResponse, Response, sse::{Event, Sse}},
    Json,
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use futures_util::stream::{self, Stream};
use playground_core_types::{Shared, shared, CoreResult, CoreError};
use crate::types::McpTool;

/// MCP request structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpRequest {
    pub id: String,
    pub method: String,
    pub params: Option<Value>,
}

/// MCP response structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpResponse {
    pub id: String,
    pub result: Option<Value>,
    pub error: Option<McpError>,
}

/// MCP error structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpError {
    pub code: i32,
    pub message: String,
    pub data: Option<Value>,
}

pub struct McpServer {
    enabled: bool,
    tools: Shared<HashMap<String, McpTool>>,
}

impl McpServer {
    pub async fn new(enabled: bool) -> CoreResult<Self> {
        Ok(Self {
            enabled,
            tools: shared(HashMap::new()),
        })
    }
    
    async fn handle_sse_request(
        State(server): State<Arc<McpServer>>,
    ) -> Sse<impl Stream<Item = Result<Event, std::convert::Infallible>>> {
        // Send initial connection event
        let initial = Event::default()
            .event("message")
            .data(json!({
                "jsonrpc": "2.0",
                "method": "connection.ready",
                "params": {}
            }).to_string());
        
        // Create stream with initial event
        let stream = stream::once(async move {
            Ok(initial)
        });
        
        Sse::new(stream)
    }
    
    async fn handle_list_tools(
        State(server): State<Arc<McpServer>>,
    ) -> Response {
        let tools = server.list_tools().await;
        
        let tools_json: Vec<Value> = tools.into_iter().map(|tool| {
            json!({
                "name": tool.name,
                "description": tool.description,
                "inputSchema": tool.parameters,
            })
        }).collect();
        
        Json(json!({
            "tools": tools_json
        })).into_response()
    }
    
    async fn handle_call_tool(
        State(server): State<Arc<McpServer>>,
        Json(request): Json<McpRequest>,
    ) -> Response {
        match server.handle_request(request).await {
            Ok(response) => Json(response).into_response(),
            Err(e) => {
                let error_response = McpResponse {
                    id: "error".to_string(),
                    result: None,
                    error: Some(McpError {
                        code: -32603,
                        message: e.to_string(),
                        data: None,
                    }),
                };
                Json(error_response).into_response()
            }
        }
    }
}

impl McpServer {
    pub async fn register_tool(&self, tool: McpTool) -> CoreResult<()> {
        if !self.enabled {
            return Ok(());
        }
        
        let mut tools = self.tools.write().await;
        
        if tools.contains_key(&tool.name) {
            return Err(CoreError::InvalidInput(format!("Tool '{}' already registered", tool.name)));
        }
        
        tools.insert(tool.name.clone(), tool);
        Ok(())
    }
    
    pub async fn unregister_tool(&self, name: &str) -> CoreResult<()> {
        let mut tools = self.tools.write().await;
        
        if tools.remove(name).is_none() {
            return Err(CoreError::NotFound(format!("Tool '{}' not found", name)));
        }
        
        Ok(())
    }
    
    pub async fn handle_request(&self, request: McpRequest) -> CoreResult<McpResponse> {
        if !self.enabled {
            return Err(CoreError::Generic("MCP server is disabled".to_string()));
        }
        
        // Parse the method to determine what to do
        match request.method.as_str() {
            "tools/list" => {
                let tools = self.list_tools().await;
                let tools_json: Vec<Value> = tools.into_iter().map(|tool| {
                    json!({
                        "name": tool.name,
                        "description": tool.description,
                        "inputSchema": tool.parameters,
                    })
                }).collect();
                
                Ok(McpResponse {
                    id: request.id,
                    result: Some(json!({ "tools": tools_json })),
                    error: None,
                })
            }
            "tools/call" => {
                // Extract tool name and arguments from params
                let params = request.params.ok_or_else(|| CoreError::InvalidInput("Missing params".to_string()))?;
                let tool_name = params["name"].as_str().ok_or_else(|| CoreError::InvalidInput("Missing tool name".to_string()))?;
                let arguments = params.get("arguments").cloned().unwrap_or(json!({}));
                
                // Get the tool
                let tools = self.tools.read().await;
                let tool = tools.get(tool_name)
                    .ok_or_else(|| CoreError::NotFound(format!("Tool '{}' not found", tool_name)))?;
                
                // In a real implementation, we would forward this to the handler channel
                // For now, return a placeholder response
                Ok(McpResponse {
                    id: request.id,
                    result: Some(json!({
                        "content": [{
                            "type": "text",
                            "text": format!("Tool '{}' called with args: {}", tool_name, arguments)
                        }]
                    })),
                    error: None,
                })
            }
            _ => {
                Err(CoreError::InvalidInput(format!("Unknown method: {}", request.method)))
            }
        }
    }
    
    pub fn router(&self) -> Router {
        Router::new()
            .route("/", get(Self::handle_sse_request))
            .route("/tools/list", post(Self::handle_list_tools))
            .route("/tools/call", post(Self::handle_call_tool))
            .with_state(Arc::new(self.clone()))
    }
    
    pub async fn list_tools(&self) -> Vec<McpTool> {
        let tools = self.tools.read().await;
        tools.values().cloned().collect()
    }
}

impl Clone for McpServer {
    fn clone(&self) -> Self {
        Self {
            enabled: self.enabled,
            tools: self.tools.clone(),
        }
    }
}