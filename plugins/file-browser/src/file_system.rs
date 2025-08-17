use std::path::{Path, PathBuf};
use std::time::SystemTime;
use tokio::sync::mpsc;
use tracing::{debug, error, warn};

use crate::file_tree::{FileSystemEntry, FileTreeEvent};

/// Handles file system operations for the file browser
pub struct FileSystemHandler {
    event_sender: mpsc::UnboundedSender<FileTreeEvent>,
}

impl FileSystemHandler {
    pub fn new(event_sender: mpsc::UnboundedSender<FileTreeEvent>) -> Self {
        Self { event_sender }
    }

    /// Load directory contents
    pub async fn load_directory(&self, path: &Path) -> Result<FileSystemEntry, std::io::Error> {
        debug!("Loading directory: {:?}", path);
        
        let mut entry = FileSystemEntry::new_directory(path.to_path_buf());
        
        // Read directory contents
        let mut entries = tokio::fs::read_dir(path).await?;
        
        while let Some(dir_entry) = entries.next_entry().await? {
                    let path = dir_entry.path();
                    let metadata = match dir_entry.metadata().await {
                        Ok(m) => m,
                        Err(e) => {
                            warn!("Failed to get metadata for {:?}: {}", path, e);
                            continue;
                        }
                    };
                    
                    let child = if metadata.is_dir() {
                        FileSystemEntry::new_directory(path)
                    } else {
                        let size = metadata.len();
                        let modified = metadata.modified().unwrap_or(SystemTime::UNIX_EPOCH);
                        FileSystemEntry::new_file(path, size, modified)
                    };
                    
                    entry.children.push(child);
                }
        }
        
        // Sort: directories first, then alphabetically
        entry.children.sort_by(|a, b| {
            match (a.is_directory, b.is_directory) {
                (true, false) => std::cmp::Ordering::Less,
                (false, true) => std::cmp::Ordering::Greater,
                _ => a.name.to_lowercase().cmp(&b.name.to_lowercase()),
            }
        });
        
        entry.is_loaded = true;
        Ok(entry)
    }

    /// Create a new file
    pub async fn create_file(&self, path: &Path) -> Result<(), std::io::Error> {
        debug!("Creating file: {:?}", path);
        tokio::fs::File::create(path).await?;
        
        let _ = self.event_sender.send(FileTreeEvent::FileCreated(path.to_path_buf()));
        Ok(())
    }

    /// Create a new directory
    pub async fn create_directory(&self, path: &Path) -> Result<(), std::io::Error> {
        debug!("Creating directory: {:?}", path);
        tokio::fs::create_dir(path).await?;
        
        let _ = self.event_sender.send(FileTreeEvent::FileCreated(path.to_path_buf()));
        Ok(())
    }

    /// Delete a file or directory
    pub async fn delete(&self, path: &Path) -> Result<(), std::io::Error> {
        debug!("Deleting: {:?}", path);
        
        let metadata = tokio::fs::metadata(path).await?;
        if metadata.is_dir() {
            tokio::fs::remove_dir_all(path).await?;
        } else {
            tokio::fs::remove_file(path).await?;
        }
        
        let _ = self.event_sender.send(FileTreeEvent::FileDeleted(path.to_path_buf()));
        Ok(())
    }

    /// Rename a file or directory
    pub async fn rename(&self, from: &Path, to: &Path) -> Result<(), std::io::Error> {
        debug!("Renaming {:?} to {:?}", from, to);
        tokio::fs::rename(from, to).await?;
        
        let _ = self.event_sender.send(FileTreeEvent::FileRenamed {
            from: from.to_path_buf(),
            to: to.to_path_buf(),
        });
        Ok(())
    }

    /// Get file stats
    pub async fn get_stats(&self, path: &Path) -> Result<FileStats, std::io::Error> {
        let metadata = tokio::fs::metadata(path).await?;
        
        Ok(FileStats {
            size: metadata.len(),
            modified: metadata.modified().unwrap_or(SystemTime::UNIX_EPOCH),
            is_directory: metadata.is_dir(),
            is_readonly: metadata.permissions().readonly(),
        })
    }
}

#[derive(Debug, Clone)]
pub struct FileStats {
    pub size: u64,
    pub modified: SystemTime,
    pub is_directory: bool,
    pub is_readonly: bool,
}

/// Watch for file system changes
/// Note: File watching is optional and may not work on all platforms
pub struct FileWatcher {
    // For now, we'll use a simple polling approach
    // TODO: Implement proper file watching with notify crate
    _paths: Vec<PathBuf>,
}

impl FileWatcher {
    pub fn new(paths: Vec<PathBuf>, _event_sender: mpsc::UnboundedSender<FileTreeEvent>) -> Result<Self, std::io::Error> {
        // For now, just store the paths
        // A real implementation would set up file watching
        Ok(Self {
            _paths: paths,
        })
    }
}