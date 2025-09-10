use std::sync::Arc;
use std::collections::HashMap;
use async_trait::async_trait;
use axum::{
    Router,
    routing::{get, post},
    extract::State,
    response::{IntoResponse, Response, sse::{Event, Sse}},
    Json,
};
use serde_json::{json, Value};
use futures_util::stream::{self, Stream};
use playground_core_server::{
    McpServerContract, McpTool, McpRequest, McpResponse, McpError
};
use playground_core_types::{Shared, shared};

pub struct McpServer {
    enabled: bool,
    tools: Shared<HashMap<String, McpTool>>,
}

impl McpServer {
    pub async fn new(enabled: bool) -> Result<Self, Box<dyn std::error::Error>> {
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
                "inputSchema": tool.input_schema,
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

#[async_trait]
impl McpServerContract for McpServer {
    async fn register_tool(&self, tool: McpTool) -> Result<(), Box<dyn std::error::Error>> {
        if !self.enabled {
            return Ok(());
        }
        
        let mut tools = self.tools.write().await;
        
        if tools.contains_key(&tool.name) {
            return Err(format!("Tool '{}' already registered", tool.name).into());
        }
        
        tools.insert(tool.name.clone(), tool);
        Ok(())
    }
    
    async fn unregister_tool(&self, name: &str) -> Result<(), Box<dyn std::error::Error>> {
        let mut tools = self.tools.write().await;
        
        if tools.remove(name).is_none() {
            return Err(format!("Tool '{}' not found", name).into());
        }
        
        Ok(())
    }
    
    async fn handle_request(&self, request: McpRequest) -> Result<McpResponse, Box<dyn std::error::Error>> {
        if !self.enabled {
            return Err("MCP server is disabled".into());
        }
        
        // Parse the method to determine what to do
        match request.method.as_str() {
            "tools/list" => {
                let tools = self.list_tools().await;
                let tools_json: Vec<Value> = tools.into_iter().map(|tool| {
                    json!({
                        "name": tool.name,
                        "description": tool.description,
                        "inputSchema": tool.input_schema,
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
                let params = request.params.ok_or("Missing params")?;
                let tool_name = params["name"].as_str().ok_or("Missing tool name")?;
                let arguments = params.get("arguments").cloned().unwrap_or(json!({}));
                
                // Get the tool
                let tools = self.tools.read().await;
                let tool = tools.get(tool_name)
                    .ok_or(format!("Tool '{}' not found", tool_name))?;
                
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
                Err(format!("Unknown method: {}", request.method).into())
            }
        }
    }
    
    fn router(&self) -> Router {
        Router::new()
            .route("/", get(Self::handle_sse_request))
            .route("/tools/list", post(Self::handle_list_tools))
            .route("/tools/call", post(Self::handle_call_tool))
            .with_state(Arc::new(self.clone()))
    }
    
    async fn list_tools(&self) -> Vec<McpTool> {
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