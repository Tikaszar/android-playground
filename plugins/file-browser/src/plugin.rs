use async_trait::async_trait;
use playground_core_plugin::Plugin;
use playground_core_types::{
    PluginMetadata, PluginId, Version, Event,
    context::Context,
    render_context::RenderContext,
    error::PluginError,
};
use playground_systems_networking::NetworkingSystem;
use playground_systems_ui::ElementGraph;
use std::path::PathBuf;
use tokio::sync::mpsc;
use tracing::{info, error, debug, warn};

use crate::file_tree::{FileTree, FileTreeEvent};
use crate::file_system::{FileSystemHandler, FileWatcher};

pub struct FileBrowserPlugin {
    metadata: PluginMetadata,
    file_tree: Option<FileTree>,
    fs_handler: Option<FileSystemHandler>,
    file_watcher: Option<FileWatcher>,
    event_receiver: Option<mpsc::UnboundedReceiver<FileTreeEvent>>,
    channel_id: Option<u16>,
    root_path: PathBuf,
}

impl FileBrowserPlugin {
    pub fn new() -> Self {
        Self {
            metadata: PluginMetadata {
                id: PluginId("file-browser".to_string()),
                name: "File Browser".to_string(),
                version: Version {
                    major: 0,
                    minor: 1,
                    patch: 0,
                },
            },
            file_tree: None,
            fs_handler: None,
            file_watcher: None,
            event_receiver: None,
            channel_id: None,
            root_path: PathBuf::from("."),
        }
    }

    pub fn with_root_path(mut self, path: PathBuf) -> Self {
        self.root_path = path;
        self
    }

    async fn initialize_file_tree(&mut self) -> Result<(), PluginError> {
        // Create event channel
        let (event_sender, event_receiver) = mpsc::unbounded_channel();
        
        // Create file tree UI
        let mut file_tree = FileTree::new(self.root_path.clone());
        file_tree.set_event_sender(event_sender.clone());
        
        // Create file system handler
        let fs_handler = FileSystemHandler::new(event_sender.clone());
        
        // Load initial directory contents
        match fs_handler.load_directory(&self.root_path).await {
            Ok(entries) => {
                file_tree.update_entries(entries);
                info!("Loaded directory: {:?}", self.root_path);
            }
            Err(e) => {
                error!("Failed to load directory {:?}: {}", self.root_path, e);
            }
        }
        
        // Set up file watcher (optional)
        match FileWatcher::new(vec![self.root_path.clone()], event_sender) {
            Ok(watcher) => {
                self.file_watcher = Some(watcher);
                info!("File watcher initialized for {:?}", self.root_path);
            }
            Err(e) => {
                warn!("File watcher not available: {}", e);
                // Continue without file watching - manual refresh will still work
            }
        }
        
        self.file_tree = Some(file_tree);
        self.fs_handler = Some(fs_handler);
        self.event_receiver = Some(event_receiver);
        
        Ok(())
    }

    async fn handle_file_tree_events(&mut self) {
        if let Some(receiver) = &mut self.event_receiver {
            // Process all pending events
            while let Ok(event) = receiver.try_recv() {
                match event {
                    FileTreeEvent::FileOpened(path) => {
                        info!("File opened: {:?}", path);
                        // Send message to editor-core plugin to open the file
                        self.send_open_file_message(path);
                    }
                    FileTreeEvent::DirectoryExpanded(path) => {
                        debug!("Directory expanded: {:?}", path);
                        // Load directory contents if not already loaded
                        self.load_directory_contents(path).await;
                    }
                    FileTreeEvent::RefreshRequested(path) => {
                        debug!("Refresh requested: {:?}", path);
                        // Reload directory or file
                        self.refresh_path(path).await;
                    }
                    _ => {
                        debug!("File tree event: {:?}", event);
                    }
                }
            }
        }
    }

    async fn load_directory_contents(&mut self, path: PathBuf) {
        if let Some(fs_handler) = &self.fs_handler {
            match fs_handler.load_directory(&path).await {
                Ok(entries) => {
                    if let Some(file_tree) = &mut self.file_tree {
                        file_tree.update_entries(entries);
                    }
                }
                Err(e) => {
                    error!("Failed to load directory {:?}: {}", path, e);
                }
            }
        }
    }

    async fn refresh_path(&mut self, path: PathBuf) {
        // Determine if path is a directory or file
        match tokio::fs::metadata(&path).await {
            Ok(metadata) => {
                if metadata.is_dir() {
                    self.load_directory_contents(path).await;
                } else {
                    // For files, refresh the parent directory
                    if let Some(parent) = path.parent() {
                        self.load_directory_contents(parent.to_path_buf()).await;
                    }
                }
            }
            Err(e) => {
                error!("Failed to get metadata for {:?}: {}", path, e);
            }
        }
    }

    fn send_open_file_message(&self, path: PathBuf) {
        // This would send a message to the editor-core plugin
        // through the networking system on our channel
        // For now, we'll just log it
        info!("Sending open file message for: {:?}", path);
        // TODO: Implement actual message sending through networking system
    }
}

#[async_trait]
impl Plugin for FileBrowserPlugin {
    fn metadata(&self) -> &PluginMetadata {
        &self.metadata
    }

    async fn on_load(&mut self, _ctx: &mut Context) -> Result<(), PluginError> {
        info!("File browser plugin loading");
        
        // Register with networking system for channels 1010-1019
        // This would normally be done through the context
        self.channel_id = Some(1010);
        
        // Initialize file tree in blocking context since on_load is sync
        let runtime = tokio::runtime::Runtime::new()
            .map_err(|e| PluginError::InitializationFailed(e.to_string()))?;
        runtime.block_on(self.initialize_file_tree())
            .map_err(|e| PluginError::InitializationFailed(e.to_string()))?;
        
        info!("File browser plugin loaded successfully");
        Ok(())
    }

    async fn on_unload(&mut self, _ctx: &mut Context) {
        info!("File browser plugin unloading");
        
        // Clean up resources
        self.file_tree = None;
        self.fs_handler = None;
        self.file_watcher = None;
        self.event_receiver = None;
    }

    async fn update(&mut self, _ctx: &mut Context, _delta_time: f32) {
        // Handle file tree events
        self.handle_file_tree_events().await;
    }

    async fn render(&mut self, _ctx: &mut RenderContext) {
        // File tree rendering is handled by the UI system
        // through the Element trait implementation
    }

    async fn on_event(&mut self, _event: &Event) -> bool {
        // Handle plugin events
        // Return true if event was handled, false otherwise
        false
    }
}

pub fn create() -> FileBrowserPlugin {
    FileBrowserPlugin::new()
}