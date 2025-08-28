use async_trait::async_trait;
use playground_systems_logic::{System, World, LogicResult, SystemsManager, Handle};
use std::path::PathBuf;
use tokio::sync::mpsc;
// Note: Using SystemsManager logging instead of tracing

use crate::file_tree::{FileTree, FileTreeEvent};
use crate::file_system::{FileSystemHandler, FileWatcher};

pub struct FileBrowserPlugin {
    file_tree: Option<FileTree>,
    fs_handler: Option<FileSystemHandler>,
    file_watcher: Option<FileWatcher>,
    event_receiver: Option<mpsc::UnboundedReceiver<FileTreeEvent>>,
    channel_id: Option<u16>,
    root_path: PathBuf,
    systems_manager: Handle<SystemsManager>,
}

impl FileBrowserPlugin {
    pub fn new(systems_manager: Handle<SystemsManager>) -> Self {
        Self {
            file_tree: None,
            fs_handler: None,
            file_watcher: None,
            event_receiver: None,
            channel_id: None,  // Will be assigned dynamically
            root_path: PathBuf::from("."),
            systems_manager,
        }
    }

    pub fn with_root_path(mut self, path: PathBuf) -> Self {
        self.root_path = path;
        self
    }

    async fn initialize_file_tree(&mut self) -> LogicResult<()> {
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
                // info!("Loaded directory: {:?}", self.root_path);
            }
            Err(e) => {
                // error!("Failed to load directory {:?}: {}", self.root_path, e);
            }
        }
        
        // Set up file watcher (optional)
        match FileWatcher::new(vec![self.root_path.clone()], event_sender) {
            Ok(watcher) => {
                self.file_watcher = Some(watcher);
                // info!("File watcher initialized for {:?}", self.root_path);
            }
            Err(e) => {
                // warn!("File watcher not available: {}", e);
                // Continue without file watching - manual refresh will still work
            }
        }
        
        self.file_tree = Some(file_tree);
        self.fs_handler = Some(fs_handler);
        self.event_receiver = Some(event_receiver);
        
        Ok(())
    }

    async fn handle_file_tree_events(&mut self) {
        // Collect events first to avoid borrow issues
        let mut events = Vec::new();
        if let Some(receiver) = &mut self.event_receiver {
            // Process all pending events
            while let Ok(event) = receiver.try_recv() {
                events.push(event);
            }
        }
        
        // Now process the collected events
        for event in events {
            match event {
                FileTreeEvent::FileOpened(path) => {
                // info!("File opened: {:?}", path);
                    // Send message to editor-core plugin to open the file
                    self.send_open_file_message(path);
                }
                FileTreeEvent::DirectoryExpanded(path) => {
                // debug!("Directory expanded: {:?}", path);
                    // Load directory contents if not already loaded
                    self.load_directory_contents(path).await;
                }
                FileTreeEvent::RefreshRequested(path) => {
                // debug!("Refresh requested: {:?}", path);
                    // Reload directory or file
                    self.refresh_path(path).await;
                }
                _ => {
                // debug!("File tree event: {:?}", event);
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
                // error!("Failed to load directory {:?}: {}", path, e);
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
                // error!("Failed to get metadata for {:?}: {}", path, e);
            }
        }
    }

    fn send_open_file_message(&self, path: PathBuf) {
        // This would send a message to the editor-core plugin
        // through the networking system on our channel
        // For now, we'll just log it
                // info!("Sending open file message for: {:?}", path);
        // TODO: Implement actual message sending through networking system
    }
}

#[async_trait]
impl System for FileBrowserPlugin {
    fn name(&self) -> &'static str {
        "FileBrowserPlugin"
    }
    
    async fn initialize(&mut self, _world: &World) -> LogicResult<()> {
        // Request dynamic channel allocation
        self.channel_id = Some(self.systems_manager.register_plugin("file-browser").await?);
        
                // info!("File Browser Plugin initialized on dynamic channel {}", self.channel_id.unwrap());
        
        // Initialize file tree
        self.initialize_file_tree().await?;
        
                // info!("File browser plugin loaded successfully");
        Ok(())
    }
    
    async fn run(&mut self, _world: &World, _delta_time: f32) -> LogicResult<()> {
        // Handle file tree events
        self.handle_file_tree_events().await;
        Ok(())
    }
    
    async fn cleanup(&mut self, _world: &World) -> LogicResult<()> {
                // info!("File browser plugin unloading");
        
        // Clean up resources
        self.file_tree = None;
        self.fs_handler = None;
        self.file_watcher = None;
        self.event_receiver = None;
        
        Ok(())
    }
}