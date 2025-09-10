use axum::{
    Router,
    routing::get,
    response::{Html, Response},
    http::StatusCode,
};
use playground_core_types::Handle;
use playground_core_server::dashboard::{Dashboard, LogLevel};
use crate::browser::BrowserBuilder;

/// WebGL server integration for serving browser pages
pub struct WebGLServerIntegration {
    browser_builder: BrowserBuilder,
    dashboard: Option<Handle<Dashboard>>,
}

impl WebGLServerIntegration {
    pub fn new() -> Self {
        Self {
            browser_builder: BrowserBuilder::new(),
            dashboard: None,
        }
    }
    
    /// Set the Dashboard for logging
    pub fn set_dashboard(&mut self, dashboard: Handle<Dashboard>) {
        self.browser_builder.set_dashboard(dashboard.clone());
        self.dashboard = Some(dashboard);
    }
    
    /// Log a message with component-specific logging
    async fn log(&self, level: LogLevel, message: String) {
        if let Some(ref dashboard) = self.dashboard {
            dashboard.log_component("systems/webgl/server", level, message, None).await;
        }
    }
    
    /// Create Axum routes for WebGL pages
    pub fn create_routes<S>(&self) -> Router<S> 
    where
        S: Clone + Send + Sync + 'static
    {
        let builder = self.browser_builder.clone();
        
        Router::new()
            .route("/", get(serve_index))
            .route("/webgl", get(serve_index))
            .route("/webgl/", get(serve_index))
            .route("/webgl/renderer.js", get(serve_renderer_js))
            .with_state(builder)
    }
}

/// Serve the main index.html
async fn serve_index(
    axum::extract::State(builder): axum::extract::State<BrowserBuilder>
) -> Html<String> {
    Html(builder.generate_index_html().await)
}

/// Serve the renderer JavaScript
async fn serve_renderer_js(
    axum::extract::State(builder): axum::extract::State<BrowserBuilder>
) -> Response {
    Response::builder()
        .status(StatusCode::OK)
        .header("Content-Type", "application/javascript")
        .body(builder.generate_renderer_js().await.into())
        .unwrap()
}