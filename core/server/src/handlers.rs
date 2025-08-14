use axum::Json;
use playground_types::server::plugin::PluginInfo;

pub async fn root() -> &'static str {
    "Android Playground Server"
}

pub async fn list_plugins() -> Json<Vec<PluginInfo>> {
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

pub async fn reload_plugin(Json(plugin_name): Json<String>) -> &'static str {
    tracing::info!("Reloading plugin: {}", plugin_name);
    "Plugin reloaded"
}