use playground_systems_ui::{
    ElementId,
    ElementBounds,
    UiResult,
};

// Stub types for compilation - these should come from systems/ui internal APIs
type Theme = ();
type RenderData = ();
type InputEvent = ();
type LayoutConstraints = ();
type LayoutResult = ();

// Stub for InputResult and related types
#[derive(Debug, Clone)]
struct InputResult {
    handled: EventHandled,
    request_focus: bool,
}

#[derive(Debug, Clone)]
enum EventHandled {
    Yes,
    No,
}

#[derive(Debug, Clone)]
enum PointerButton {
    Primary,
    Secondary,
    Middle,
}
use nalgebra::{Vector2, Vector4};
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use tokio::sync::mpsc;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub enum FileTreeEvent {
    FileSelected(PathBuf),
    FileOpened(PathBuf),
    DirectoryExpanded(PathBuf),
    DirectoryCollapsed(PathBuf),
    RefreshRequested(PathBuf),
    FileCreated(PathBuf),
    FileRenamed { from: PathBuf, to: PathBuf },
    FileDeleted(PathBuf),
}

#[derive(Debug, Clone)]
pub struct FileSystemEntry {
    pub path: PathBuf,
    pub name: String,
    pub is_directory: bool,
    pub size: Option<u64>,
    pub modified: Option<std::time::SystemTime>,
    pub children: Vec<FileSystemEntry>,
    pub is_loaded: bool,
}

impl FileSystemEntry {
    pub fn new_directory(path: PathBuf) -> Self {
        let name = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("/")
            .to_string();
        
        Self {
            path,
            name,
            is_directory: true,
            size: None,
            modified: None,
            children: Vec::new(),
            is_loaded: false,
        }
    }

    pub fn new_file(path: PathBuf, size: u64, modified: std::time::SystemTime) -> Self {
        let name = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown")
            .to_string();
        
        Self {
            path,
            name,
            is_directory: false,
            size: Some(size),
            modified: Some(modified),
            children: Vec::new(),
            is_loaded: true,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ViewMode {
    List,
    Icon,
}

pub struct FileTree {
    id: Uuid,
    position: Vector2<f32>,
    size: Vector2<f32>,
    root_path: PathBuf,
    root_entry: FileSystemEntry,
    expanded_paths: HashSet<PathBuf>,
    selected_path: Option<PathBuf>,
    hovered_path: Option<PathBuf>,
    view_mode: ViewMode,
    show_hidden: bool,
    item_height: f32,
    indent_width: f32,
    icon_size: f32,
    scroll_offset: f32,
    visible_items: Vec<(PathBuf, usize)>, // (path, depth)
    event_sender: Option<mpsc::UnboundedSender<FileTreeEvent>>,
    theme: (),  // Theme stub
    filter_pattern: Option<String>,
    children: Vec<ElementId>,
    dirty: bool,
    visible: bool,
}

impl FileTree {
    pub fn new(root_path: PathBuf) -> Self {
        let root_entry = FileSystemEntry::new_directory(root_path.clone());
        
        Self {
            id: Uuid::new_v4(),
            position: Vector2::zeros(),
            size: Vector2::zeros(),
            root_path: root_path.clone(),
            root_entry,
            expanded_paths: HashSet::from([root_path.clone()]),
            selected_path: None,
            hovered_path: None,
            view_mode: ViewMode::List,
            show_hidden: false,
            item_height: 24.0,
            indent_width: 20.0,
            icon_size: 16.0,
            scroll_offset: 0.0,
            visible_items: Vec::new(),
            event_sender: None,
            theme: (),  // Theme stub
            filter_pattern: None,
            children: Vec::new(),
            dirty: true,
            visible: true,
        }
    }

    pub fn set_theme(&mut self, theme: ()) {
        self.theme = theme;
        self.dirty = true;
    }

    pub fn set_event_sender(&mut self, sender: mpsc::UnboundedSender<FileTreeEvent>) {
        self.event_sender = Some(sender);
    }

    pub fn set_view_mode(&mut self, mode: ViewMode) {
        self.view_mode = mode;
        self.update_visible_items();
        self.dirty = true;
    }

    pub fn set_show_hidden(&mut self, show: bool) {
        self.show_hidden = show;
        self.update_visible_items();
        self.dirty = true;
    }

    pub fn set_filter(&mut self, pattern: Option<String>) {
        self.filter_pattern = pattern;
        self.update_visible_items();
        self.dirty = true;
    }

    pub fn expand_path(&mut self, path: &Path) {
        self.expanded_paths.insert(path.to_path_buf());
        self.update_visible_items();
        self.dirty = true;
        
        if let Some(sender) = &self.event_sender {
            let _ = sender.send(FileTreeEvent::DirectoryExpanded(path.to_path_buf()));
        }
    }

    pub fn collapse_path(&mut self, path: &Path) {
        self.expanded_paths.remove(path);
        self.update_visible_items();
        self.dirty = true;
        
        if let Some(sender) = &self.event_sender {
            let _ = sender.send(FileTreeEvent::DirectoryCollapsed(path.to_path_buf()));
        }
    }

    pub fn toggle_path(&mut self, path: &Path) {
        if self.expanded_paths.contains(path) {
            self.collapse_path(path);
        } else {
            self.expand_path(path);
        }
    }

    pub fn select_path(&mut self, path: Option<PathBuf>) {
        self.selected_path = path.clone();
        self.dirty = true;
        
        if let Some(path) = path {
            if let Some(sender) = &self.event_sender {
                let _ = sender.send(FileTreeEvent::FileSelected(path));
            }
        }
    }

    pub fn open_file(&mut self, path: &Path) {
        if let Some(sender) = &self.event_sender {
            let _ = sender.send(FileTreeEvent::FileOpened(path.to_path_buf()));
        }
    }

    pub fn update_entries(&mut self, entry: FileSystemEntry) {
        if self.root_entry.path == entry.path {
            self.root_entry = entry;
        } else {
            self.merge_entry_recursive(&mut self.root_entry.clone(), entry);
        }
        self.update_visible_items();
        self.dirty = true;
    }

    fn merge_entry_recursive(&mut self, target: &mut FileSystemEntry, source: FileSystemEntry) {
        if target.path == source.path {
            target.children = source.children;
            target.is_loaded = source.is_loaded;
            target.size = source.size;
            target.modified = source.modified;
        } else {
            for child in &mut target.children {
                if child.path == source.path {
                    *child = source;
                    return;
                }
                self.merge_entry_recursive(child, source.clone());
            }
        }
    }

    fn update_visible_items(&mut self) {
        self.visible_items.clear();
        let root_entry = self.root_entry.clone();
        self.collect_visible_items(&root_entry, 0);
    }

    fn collect_visible_items(&mut self, entry: &FileSystemEntry, depth: usize) {
        if !self.should_show_entry(entry) {
            return;
        }

        self.visible_items.push((entry.path.clone(), depth));

        if entry.is_directory && self.expanded_paths.contains(&entry.path) {
            for child in &entry.children {
                self.collect_visible_items(child, depth + 1);
            }
        }
    }

    fn should_show_entry(&self, entry: &FileSystemEntry) -> bool {
        if !self.show_hidden && entry.name.starts_with('.') {
            return false;
        }

        if let Some(pattern) = &self.filter_pattern {
            if !entry.name.to_lowercase().contains(&pattern.to_lowercase()) {
                return false;
            }
        }

        true
    }

    fn get_file_icon(&self, _entry: &FileSystemEntry) -> &'static str {
        // Icons removed for now as text rendering is not implemented
        // Will be replaced with actual icon rendering later
        ""
    }

    fn render_list_view(&self, _data: &mut RenderData) {
        // TODO: Implement once RenderData is properly exposed from systems/ui
        /*
        let visible_start = (self.scroll_offset / self.item_height) as usize;
        let visible_end = ((self.scroll_offset + self.size.y) / self.item_height) as usize + 1;

        for (index, (path, _depth)) in self.visible_items.iter().enumerate() {
            if index < visible_start || index > visible_end {
                continue;
            }

            let y = index as f32 * self.item_height - self.scroll_offset;

            let is_selected = self.selected_path.as_ref() == Some(path);
            let is_hovered = self.hovered_path.as_ref() == Some(path);

            if is_selected {
                data.add_quad(
                    Vector2::new(self.position.x, self.position.y + y),
                    Vector2::new(self.size.x, self.item_height),
                    self.theme.colors.highlight,
                );
            } else if is_hovered {
                data.add_quad(
                    Vector2::new(self.position.x, self.position.y + y),
                    Vector2::new(self.size.x, self.item_height),
                    Vector4::new(
                        self.theme.colors.highlight.x,
                        self.theme.colors.highlight.y,
                        self.theme.colors.highlight.z,
                        0.5,
                    ),
                );
            }

            // Text rendering would go here once implemented
            // For now just render colored rectangles
        }
        */
    }

    fn render_icon_view(&self, _data: &mut RenderData) {
        // TODO: Implement once RenderData is properly exposed from systems/ui
        /*
        let cols = (self.size.x / 80.0).max(1.0) as usize;
        let icon_size = 64.0;
        let item_size = 92.0;

        for (index, (path, _)) in self.visible_items.iter().enumerate() {
            let col = index % cols;
            let row = index / cols;
            
            let x = col as f32 * item_size;
            let y = row as f32 * item_size - self.scroll_offset;

            if y + item_size < 0.0 || y > self.size.y {
                continue;
            }

            let is_selected = self.selected_path.as_ref() == Some(path);
            let is_hovered = self.hovered_path.as_ref() == Some(path);

            if is_selected || is_hovered {
                let color = if is_selected {
                    self.theme.colors.highlight
                } else {
                    Vector4::new(
                        self.theme.colors.highlight.x,
                        self.theme.colors.highlight.y,
                        self.theme.colors.highlight.z,
                        0.5,
                    )
                };
                
                data.add_quad(
                    Vector2::new(
                        self.position.x + x + (item_size - icon_size) / 2.0 - 4.0,
                        self.position.y + y,
                    ),
                    Vector2::new(icon_size + 8.0, item_size),
                    color,
                );
            }

            // Icon and text rendering would go here once implemented
        }
        */
    }

    fn find_entry(&self, path: &Path) -> Option<&FileSystemEntry> {
        self.find_entry_recursive(&self.root_entry, path)
    }

    fn find_entry_recursive<'a>(&self, entry: &'a FileSystemEntry, path: &Path) -> Option<&'a FileSystemEntry> {
        if entry.path == path {
            return Some(entry);
        }

        for child in &entry.children {
            if let Some(found) = self.find_entry_recursive(child, path) {
                return Some(found);
            }
        }

        None
    }

    fn handle_click(&mut self, position: Vector2<f32>) -> InputResult {
        let relative_pos = position - self.position;
        
        match self.view_mode {
            ViewMode::List => {
                let index = ((relative_pos.y + self.scroll_offset) / self.item_height) as usize;
                if let Some((path, _)) = self.visible_items.get(index).cloned() {
                    if let Some(entry) = self.find_entry(&path) {
                        if entry.is_directory {
                            self.toggle_path(&path);
                        } else {
                            self.open_file(&path);
                        }
                        self.select_path(Some(path));
                    }
                    return InputResult { handled: EventHandled::Yes, request_focus: false };
                }
            }
            ViewMode::Icon => {
                let cols = (self.size.x / 80.0).max(1.0) as usize;
                let item_size = 92.0;
                
                let col = (relative_pos.x / item_size) as usize;
                let row = ((relative_pos.y + self.scroll_offset) / item_size) as usize;
                let index = row * cols + col;
                
                if let Some((path, _)) = self.visible_items.get(index).cloned() {
                    if let Some(entry) = self.find_entry(&path) {
                        if entry.is_directory {
                            self.toggle_path(&path);
                        } else {
                            self.open_file(&path);
                        }
                        self.select_path(Some(path));
                    }
                    return InputResult { handled: EventHandled::Yes, request_focus: false };
                }
            }
        }
        
        InputResult { handled: EventHandled::No, request_focus: false }
    }

    fn handle_scroll(&mut self, delta: f32) -> InputResult {
        let max_scroll = match self.view_mode {
            ViewMode::List => {
                (self.visible_items.len() as f32 * self.item_height - self.size.y).max(0.0)
            }
            ViewMode::Icon => {
                let cols = (self.size.x / 80.0).max(1.0) as usize;
                let rows = (self.visible_items.len() + cols - 1) / cols;
                (rows as f32 * 92.0 - self.size.y).max(0.0)
            }
        };
        
        self.scroll_offset = (self.scroll_offset - delta * 20.0).clamp(0.0, max_scroll);
        self.dirty = true;
        InputResult { handled: EventHandled::Yes, request_focus: false }
    }
}

/* TODO: Element trait not exposed from systems/ui
impl Element for FileTree {
    fn id(&self) -> Uuid {
        self.id
    }

    fn type_name(&self) -> &str {
        "FileTree"
    }

    fn layout(&mut self, constraints: &LayoutConstraints) -> UiResult<LayoutResult> {
        self.size = constraints.available_size;
        Ok(LayoutResult::new(self.size, self.position))
    }

    fn handle_input(&mut self, event: &InputEvent) -> InputResult {
        match event {
            InputEvent::PointerDown { position, button: PointerButton::Primary } => {
                self.handle_click(*position)
            }
            InputEvent::Scroll { delta, .. } => {
                self.handle_scroll(delta.y)
            }
            InputEvent::PointerMove { position, .. } => {
                let relative_pos = position - self.position;
                
                match self.view_mode {
                    ViewMode::List => {
                        let index = ((relative_pos.y + self.scroll_offset) / self.item_height) as usize;
                        if let Some((path, _)) = self.visible_items.get(index) {
                            self.hovered_path = Some(path.clone());
                        } else {
                            self.hovered_path = None;
                        }
                    }
                    ViewMode::Icon => {
                        let cols = (self.size.x / 80.0).max(1.0) as usize;
                        let item_size = 92.0;
                        
                        let col = (relative_pos.x / item_size) as usize;
                        let row = ((relative_pos.y + self.scroll_offset) / item_size) as usize;
                        let index = row * cols + col;
                        
                        if let Some((path, _)) = self.visible_items.get(index) {
                            self.hovered_path = Some(path.clone());
                        } else {
                            self.hovered_path = None;
                        }
                    }
                }
                
                self.dirty = true;
                InputResult { handled: EventHandled::Yes, request_focus: false }
            }
            _ => InputResult { handled: EventHandled::No, request_focus: false },
        }
    }

    fn render(&self, _theme: &Theme) -> UiResult<RenderData> {
        let mut data = RenderData::new();
        
        // Background
        data.add_quad(
            self.position,
            self.size,
            self.theme.colors.background,
        );

        match self.view_mode {
            ViewMode::List => self.render_list_view(&mut data),
            ViewMode::Icon => self.render_icon_view(&mut data),
        }
        
        Ok(data)
    }

    fn update(&mut self, _delta_time: f32) {
        // Could add smooth scrolling animation here
    }

    fn children(&self) -> &[ElementId] {
        &self.children
    }

    fn children_mut(&mut self) -> &mut Vec<ElementId> {
        &mut self.children
    }

    fn is_dirty(&self) -> bool {
        self.dirty
    }

    fn mark_clean(&mut self) {
        self.dirty = false;
    }

    fn mark_dirty(&mut self) {
        self.dirty = true;
    }

    fn bounds(&self) -> ElementBounds {
        ElementBounds {
            position: self.position,
            size: self.size,
        }
    }

    fn set_bounds(&mut self, bounds: ElementBounds) {
        self.position = bounds.position;
        self.size = bounds.size;
        self.update_visible_items();
    }

    fn is_visible(&self) -> bool {
        self.visible
    }

    fn set_visible(&mut self, visible: bool) {
        self.visible = visible;
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}
*/