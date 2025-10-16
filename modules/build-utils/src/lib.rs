use sha2::{Digest, Sha256};
use std::fs;
use std::path::Path;
use walkdir::WalkDir;

pub fn hash_directory(dir: &Path) -> u32 {
    let mut hasher = Sha256::new();

    let mut entries: Vec<_> = WalkDir::new(dir)
        .follow_links(false)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
        .collect();

    entries.sort_by_key(|e| e.path().to_path_buf());

    for entry in entries {
        if let Ok(contents) = fs::read(entry.path()) {
            hasher.update(&contents);
        }
    }

    let result = hasher.finalize();
    u32::from_be_bytes([result[0], result[1], result[2], result[3]])
}

pub fn generate_api_version(manifest_dir: &str) -> u32 {
    let view_dir = Path::new(manifest_dir).join("src/view");
    if view_dir.exists() {
        hash_directory(&view_dir)
    } else {
        0
    }
}

pub fn generate_state_version(manifest_dir: &str) -> u32 {
    let model_dir = Path::new(manifest_dir).join("src/model");
    if model_dir.exists() {
        hash_directory(&model_dir)
    } else {
        0
    }
}
