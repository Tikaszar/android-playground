use serde::{Deserialize, Serialize};

/// Text buffer for editing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextBuffer {
    pub path: String,
    pub content: Vec<String>, // Lines of text
    pub version: u32,
    pub modified: bool,
    pub language: String,
}

impl TextBuffer {
    pub fn new(path: String, content: String) -> Self {
        let lines: Vec<String> = if content.is_empty() {
            vec![String::new()]
        } else {
            content.lines().map(|s| s.to_string()).collect()
        };
        
        let language = Self::detect_language(&path);
        
        Self {
            path,
            content: lines,
            version: 0,
            modified: false,
            language,
        }
    }
    
    /// Get line count
    pub fn line_count(&self) -> usize {
        self.content.len()
    }
    
    /// Get a line by index
    pub fn get_line(&self, line: usize) -> Option<&str> {
        self.content.get(line).map(|s| s.as_str())
    }
    
    /// Insert text at position
    pub fn insert(&mut self, line: usize, column: usize, text: &str) {
        if line >= self.content.len() {
            return;
        }
        
        let current_line = &mut self.content[line];
        
        if text.contains('\n') {
            // Multi-line insert
            let lines: Vec<&str> = text.split('\n').collect();
            let (before, after) = current_line.split_at(column.min(current_line.len()));
            
            // First line gets the before part + first new line
            let first_line = format!("{}{}", before, lines[0]);
            
            // Last line gets last new line + after part
            let last_line = format!("{}{}", lines[lines.len() - 1], after);
            
            // Build new content
            let mut new_content = Vec::new();
            
            // Add lines before insertion
            for i in 0..line {
                new_content.push(self.content[i].clone());
            }
            
            // Add first line
            new_content.push(first_line);
            
            // Add middle lines
            for i in 1..lines.len() - 1 {
                new_content.push(lines[i].to_string());
            }
            
            // Add last line if different from first
            if lines.len() > 1 {
                new_content.push(last_line);
            }
            
            // Add lines after insertion
            for i in line + 1..self.content.len() {
                new_content.push(self.content[i].clone());
            }
            
            self.content = new_content;
        } else {
            // Single line insert
            current_line.insert_str(column.min(current_line.len()), text);
        }
        
        self.version += 1;
        self.modified = true;
    }
    
    /// Delete text in range
    pub fn delete(&mut self, start_line: usize, start_col: usize, end_line: usize, end_col: usize) {
        if start_line >= self.content.len() || end_line >= self.content.len() {
            return;
        }
        
        if start_line == end_line {
            // Single line delete
            let line = &mut self.content[start_line];
            let start = start_col.min(line.len());
            let end = end_col.min(line.len());
            line.replace_range(start..end, "");
        } else {
            // Multi-line delete
            let start_text = self.content[start_line][..start_col.min(self.content[start_line].len())].to_string();
            let end_text = self.content[end_line][end_col.min(self.content[end_line].len())..].to_string();
            
            let merged_line = format!("{}{}", start_text, end_text);
            
            // Build new content
            let mut new_content = Vec::new();
            
            // Add lines before deletion
            for i in 0..start_line {
                new_content.push(self.content[i].clone());
            }
            
            // Add merged line
            new_content.push(merged_line);
            
            // Add lines after deletion
            for i in end_line + 1..self.content.len() {
                new_content.push(self.content[i].clone());
            }
            
            self.content = new_content;
        }
        
        self.version += 1;
        self.modified = true;
    }
    
    /// Get full text content
    pub fn get_text(&self) -> String {
        self.content.join("\n")
    }
    
    /// Replace entire content
    pub fn set_text(&mut self, text: String) {
        self.content = if text.is_empty() {
            vec![String::new()]
        } else {
            text.lines().map(|s| s.to_string()).collect()
        };
        
        self.version += 1;
        self.modified = true;
    }
    
    /// Mark as saved
    pub fn mark_saved(&mut self) {
        self.modified = false;
    }
    
    fn detect_language(path: &str) -> String {
        match path.split('.').last() {
            Some("rs") => "rust",
            Some("js") => "javascript",
            Some("ts") => "typescript",
            Some("py") => "python",
            Some("go") => "go",
            Some("java") => "java",
            Some("cpp") | Some("cc") | Some("cxx") => "cpp",
            Some("c") | Some("h") => "c",
            Some("md") => "markdown",
            Some("json") => "json",
            Some("toml") => "toml",
            Some("yaml") | Some("yml") => "yaml",
            _ => "text",
        }.to_string()
    }
}