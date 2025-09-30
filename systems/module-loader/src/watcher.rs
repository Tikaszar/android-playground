//! File watcher for hot-reload functionality

use std::sync::Arc;
use std::path::Path;
use tokio::sync::mpsc;
use notify::{Watcher, RecursiveMode, Event, EventKind, RecommendedWatcher};
use tracing::{info, warn, error};
use playground_core_types::{CoreResult, CoreError};

/// File watcher for module hot-reload
pub struct ModuleWatcher {
    watcher: RecommendedWatcher,
    _shutdown_tx: mpsc::Sender<()>,
}

impl ModuleWatcher {
    /// Create a new module watcher
    pub fn new(loader: Arc<super::ModuleLoader>) -> CoreResult<Self> {
        let (tx, mut rx) = mpsc::channel(100);
        let (shutdown_tx, mut shutdown_rx) = mpsc::channel::<()>(1);

        // Create file watcher
        let mut watcher = notify::recommended_watcher(move |res: Result<Event, notify::Error>| {
            match res {
                Ok(event) => {
                    if let Err(e) = tx.blocking_send(event) {
                        error!("Failed to send file event: {}", e);
                    }
                }
                Err(e) => error!("File watcher error: {}", e),
            }
        }).map_err(|e| CoreError::Generic(format!("Failed to create file watcher: {}", e)))?;

        // Watch module directories
        let watch_paths = vec![
            Path::new("target/debug"),
            Path::new("target/release"),
            Path::new("modules"),
        ];

        for path in &watch_paths {
            if path.exists() {
                watcher.watch(path, RecursiveMode::Recursive)
                    .map_err(|e| CoreError::Generic(format!(
                        "Failed to watch directory {}: {}",
                        path.display(),
                        e
                    )))?;
                info!("Watching directory for changes: {}", path.display());
            }
        }

        // Spawn handler task
        tokio::spawn(async move {
            loop {
                tokio::select! {
                    Some(event) = rx.recv() => {
                        handle_file_event(event, loader.clone()).await;
                    }
                    _ = shutdown_rx.recv() => {
                        info!("Module watcher shutting down");
                        break;
                    }
                }
            }
        });

        Ok(Self {
            watcher,
            _shutdown_tx: shutdown_tx,
        })
    }
}

/// Handle file system events
async fn handle_file_event(event: Event, loader: Arc<super::ModuleLoader>) {
    match event.kind {
        EventKind::Modify(_) | EventKind::Create(_) => {
            for path in event.paths {
                // Check if it's a module file
                if is_module_file(&path) {
                    if let Some(module_name) = extract_module_name(&path) {
                        info!("Detected change in module: {}", module_name);

                        // Check if module is loaded
                        let modules = loader.list_modules().await;
                        if modules.contains(&module_name) {
                            // Hot-reload the module
                            match loader.hot_reload(&module_name).await {
                                Ok(()) => info!("Hot-reloaded module: {}", module_name),
                                Err(e) => error!("Failed to hot-reload module {}: {}", module_name, e),
                            }
                        } else {
                            // New module, try to load it
                            match loader.load_module(&path).await {
                                Ok(()) => info!("Loaded new module: {}", module_name),
                                Err(e) => warn!("Failed to load new module {}: {}", module_name, e),
                            }
                        }
                    }
                }
            }
        }
        EventKind::Remove(_) => {
            for path in event.paths {
                if is_module_file(&path) {
                    if let Some(module_name) = extract_module_name(&path) {
                        info!("Module file removed: {}", module_name);
                        // Note: We don't auto-unload modules when files are removed
                        // This prevents accidental unloads during rebuilds
                    }
                }
            }
        }
        _ => {}
    }
}

/// Check if a path is a module file
fn is_module_file(path: &Path) -> bool {
    if let Some(ext) = path.extension() {
        ext == std::env::consts::DLL_EXTENSION
    } else {
        false
    }
}

/// Extract module name from file path
fn extract_module_name(path: &Path) -> Option<String> {
    path.file_stem()
        .and_then(|s| s.to_str())
        .map(|s| {
            // Remove lib prefix if present (Unix convention)
            if s.starts_with("lib") {
                s.strip_prefix("lib").unwrap_or(s).to_string()
            } else {
                s.to_string()
            }
        })
        .map(|s| {
            // Convert underscores to slashes for module paths
            // e.g., playground_core_ecs -> core/ecs
            if s.starts_with("playground_") {
                s.strip_prefix("playground_")
                    .unwrap_or(&s)
                    .replace('_', "/")
            } else {
                s
            }
        })
}