use axum::{
    routing::{get, post},
    Json, Router,
};
use playground_types::server::plugin::PluginInfo;
use std::net::SocketAddr;
use tower_http::cors::CorsLayer;
use tracing_subscriber;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();

    let app = Router::new()
        .route("/", get(root))
        .route("/api/plugins", get(list_plugins))
        .route("/api/reload", post(reload_plugin))
        .layer(CorsLayer::permissive());

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    tracing::info!("Server listening on {}", addr);
    
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;
    
    Ok(())
}

async fn root() -> &'static str {
    "Android Playground Server"
}

async fn list_plugins() -> Json<Vec<PluginInfo>> {
    let plugins = vec![
        PluginInfo {
            name: "idle-game".to_string(),
            version: "0.1.0".to_string(),
            description: "An idle game plugin.".to_string(),
        },
        PluginInfo {
            name: "playground-editor".to_string(),
            version: "0.1.0".to_string(),
            description: "A browser-based editor for the playground.".to_string(),
        },
    ];
    Json(plugins)
}

async fn reload_plugin(Json(plugin_name): Json<String>) -> &'static str {
    tracing::info!("Reloading plugin: {}", plugin_name);
    "Plugin reloaded"
}
