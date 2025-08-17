//! VSCode/Godot-style docking system for draggable, resizable panels

use crate::{UiResult, UiError, Element, ElementId, ElementBounds};
use crate::layout::{LayoutConstraints, LayoutResult};
use crate::input::{InputEvent, InputResult, EventHandled};
use crate::rendering::RenderData;
use crate::theme::Theme;
use nalgebra::{Vector2, Vector4};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use std::collections::HashMap;

/// Docking orientation for panel splits
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DockOrientation {
    Horizontal, // Split left/right
    Vertical,   // Split top/bottom
}

/// Docking position for dragged panels
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DockPosition {
    Left,
    Right,
    Top,
    Bottom,
    Center, // Creates tabs
}

/// Configuration for a dock node
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DockConfig {
    /// Unique ID for this dock
    pub id: Uuid,
    /// Relative size (0.0 to 1.0)
    pub size: f32,
    /// Minimum size in pixels
    pub min_size: f32,
    /// Whether this dock can be closed
    pub closable: bool,
    /// Whether this dock is resizable
    pub resizable: bool,
}

impl Default for DockConfig {
    fn default() -> Self {
        Self {
            id: Uuid::new_v4(),
            size: 0.5,
            min_size: 100.0,
            closable: true,
            resizable: true,
        }
    }
}

/// A single dock node in the docking tree
#[derive(Debug, Clone)]
pub enum DockNode {
    /// Container that holds split panels
    Container {
        config: DockConfig,
        orientation: DockOrientation,
        children: Vec<DockNode>,
    },
    /// Tab container for multiple panels
    TabContainer {
        config: DockConfig,
        tabs: Vec<TabInfo>,
        active_tab: usize,
    },
    /// Empty dock space
    Empty {
        config: DockConfig,
    },
}

/// Information about a tab
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TabInfo {
    pub id: Uuid,
    pub title: String,
    pub element_id: Uuid, // ID of the element in the panel
    pub closable: bool,
    pub icon: Option<String>, // Icon identifier
}

/// Screen orientation for responsive design
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ScreenOrientation {
    Portrait,
    Landscape,
}

/// Responsive layout configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResponsiveConfig {
    /// Portrait mode layout
    pub portrait_layout: Option<SavedLayout>,
    /// Landscape mode layout
    pub landscape_layout: Option<SavedLayout>,
    /// Breakpoint width for orientation switch
    pub breakpoint_width: f32,
    /// Auto-collapse panels in portrait mode
    pub auto_collapse_portrait: bool,
    /// Minimum panel size multiplier for mobile
    pub mobile_size_multiplier: f32,
}

impl Default for ResponsiveConfig {
    fn default() -> Self {
        Self {
            portrait_layout: None,
            landscape_layout: None,
            breakpoint_width: 768.0,
            auto_collapse_portrait: true,
            mobile_size_multiplier: 1.5,
        }
    }
}

/// Main docking layout manager
pub struct DockingLayout {
    id: Uuid,
    root: DockNode,
    bounds: ElementBounds,
    dirty: bool,
    visible: bool,
    
    // Drag state
    dragging: Option<DragState>,
    resize_handles: Vec<ResizeHandle>,
    hover_position: Option<DockPosition>,
    
    // Layout cache
    panel_bounds: HashMap<Uuid, ElementBounds>,
    
    // Configuration
    handle_width: f32,
    tab_height: f32,
    drop_preview_alpha: f32,
    
    // Responsive design
    responsive_config: ResponsiveConfig,
    current_orientation: ScreenOrientation,
}

/// State while dragging a panel
#[derive(Debug, Clone)]
struct DragState {
    panel_id: Uuid,
    start_position: Vector2<f32>,
    current_position: Vector2<f32>,
    original_dock: Uuid,
}

/// Resize handle between panels
#[derive(Debug, Clone)]
struct ResizeHandle {
    id: Uuid,
    bounds: ElementBounds,
    orientation: DockOrientation,
    left_dock: Uuid,
    right_dock: Uuid,
    hovering: bool,
    dragging: bool,
}

impl DockingLayout {
    /// Create a new docking layout
    pub fn new() -> Self {
        Self {
            id: Uuid::new_v4(),
            root: DockNode::Empty {
                config: DockConfig::default(),
            },
            bounds: ElementBounds::new(0.0, 0.0, 800.0, 600.0),
            dirty: true,
            visible: true,
            dragging: None,
            resize_handles: Vec::new(),
            hover_position: None,
            panel_bounds: HashMap::new(),
            handle_width: 4.0,
            tab_height: 30.0,
            drop_preview_alpha: 0.3,
            responsive_config: ResponsiveConfig::default(),
            current_orientation: ScreenOrientation::Landscape,
        }
    }
    
    /// Update orientation based on screen size
    pub fn update_orientation(&mut self, width: f32, height: f32) {
        let new_orientation = if width < self.responsive_config.breakpoint_width || height > width {
            ScreenOrientation::Portrait
        } else {
            ScreenOrientation::Landscape
        };
        
        if new_orientation != self.current_orientation {
            self.switch_orientation(new_orientation);
        }
    }
    
    /// Switch to a different orientation layout
    fn switch_orientation(&mut self, orientation: ScreenOrientation) {
        // Save current layout for the old orientation
        let current_layout = self.save_layout().ok();
        
        match self.current_orientation {
            ScreenOrientation::Portrait => {
                if let Some(layout) = current_layout {
                    if let Ok(saved) = serde_json::from_str::<SavedLayout>(&layout) {
                        self.responsive_config.portrait_layout = Some(saved);
                    }
                }
            }
            ScreenOrientation::Landscape => {
                if let Some(layout) = current_layout {
                    if let Ok(saved) = serde_json::from_str::<SavedLayout>(&layout) {
                        self.responsive_config.landscape_layout = Some(saved);
                    }
                }
            }
        }
        
        // Load layout for new orientation
        self.current_orientation = orientation;
        
        match orientation {
            ScreenOrientation::Portrait => {
                if let Some(ref layout) = self.responsive_config.portrait_layout {
                    self.root = self.deserialize_node(&layout.root);
                } else if self.responsive_config.auto_collapse_portrait {
                    // Auto-collapse to tabs in portrait mode
                    self.collapse_to_tabs();
                }
            }
            ScreenOrientation::Landscape => {
                if let Some(ref layout) = self.responsive_config.landscape_layout {
                    self.root = self.deserialize_node(&layout.root);
                }
            }
        }
        
        self.mark_dirty();
    }
    
    /// Collapse all panels into a single tab container for mobile
    fn collapse_to_tabs(&mut self) {
        let mut all_tabs = Vec::new();
        self.collect_all_tabs(&self.root.clone(), &mut all_tabs);
        
        if !all_tabs.is_empty() {
            self.root = DockNode::TabContainer {
                config: DockConfig::default(),
                tabs: all_tabs,
                active_tab: 0,
            };
        }
    }
    
    fn collect_all_tabs(&self, node: &DockNode, tabs: &mut Vec<TabInfo>) {
        match node {
            DockNode::TabContainer { tabs: node_tabs, .. } => {
                tabs.extend_from_slice(node_tabs);
            }
            DockNode::Container { children, .. } => {
                for child in children {
                    self.collect_all_tabs(child, tabs);
                }
            }
            _ => {}
        }
    }
    
    /// Save responsive configuration
    pub fn save_responsive_config(&self) -> UiResult<String> {
        serde_json::to_string(&self.responsive_config)
            .map_err(|e| UiError::Other(format!("Failed to serialize responsive config: {}", e)))
    }
    
    /// Load responsive configuration
    pub fn load_responsive_config(&mut self, json: &str) -> UiResult<()> {
        self.responsive_config = serde_json::from_str(json)
            .map_err(|e| UiError::Other(format!("Failed to deserialize responsive config: {}", e)))?;
        Ok(())
    }
    
    /// Split a dock horizontally or vertically
    pub fn split_dock(&mut self, dock_id: Uuid, orientation: DockOrientation, ratio: f32) -> UiResult<(Uuid, Uuid)> {
        let new_left = Uuid::new_v4();
        let new_right = Uuid::new_v4();
        
        let mut root = std::mem::replace(&mut self.root, DockNode::Empty { config: DockConfig::default() });
        let result = Self::split_node_recursive(&mut root, dock_id, orientation, ratio, new_left, new_right);
        self.root = root;
        result?;
        self.mark_dirty();
        
        Ok((new_left, new_right))
    }
    
    fn split_node_recursive(
        node: &mut DockNode,
        target_id: Uuid,
        orientation: DockOrientation,
        ratio: f32,
        new_left: Uuid,
        new_right: Uuid,
    ) -> UiResult<bool> {
        match node {
            DockNode::Empty { config } if config.id == target_id => {
                // Replace empty node with container
                let left_config = DockConfig {
                    id: new_left,
                    size: ratio,
                    ..config.clone()
                };
                let right_config = DockConfig {
                    id: new_right,
                    size: 1.0 - ratio,
                    ..config.clone()
                };
                
                *node = DockNode::Container {
                    config: config.clone(),
                    orientation,
                    children: vec![
                        DockNode::Empty { config: left_config },
                        DockNode::Empty { config: right_config },
                    ],
                };
                Ok(true)
            }
            DockNode::TabContainer { .. } => {
                // Check if this is the target node
                let is_target = if let DockNode::TabContainer { config, .. } = node {
                    config.id == target_id
                } else {
                    false
                };
                
                if is_target {
                    // Get config before replacing
                    let orig_config = if let DockNode::TabContainer { config, .. } = node {
                        config.clone()
                    } else {
                        return Ok(false);
                    };
                    
                    // Convert tab container to split container
                    let left_config = DockConfig {
                        id: new_left,
                        size: ratio,
                        ..orig_config.clone()
                    };
                    let right_config = DockConfig {
                        id: new_right,
                        size: 1.0 - ratio,
                        ..orig_config.clone()
                    };
                    
                    let old_node = std::mem::replace(node, DockNode::Empty { config: orig_config.clone() });
                    *node = DockNode::Container {
                        config: orig_config,
                        orientation,
                        children: vec![
                            old_node,
                            DockNode::Empty { config: right_config },
                        ],
                    };
                    
                    // Update the ID of the moved node
                    if let DockNode::Container { children, .. } = node {
                        if let Some(DockNode::TabContainer { config, .. }) = children.get_mut(0) {
                            config.id = new_left;
                        }
                    }
                    Ok(true)
                } else {
                    Ok(false)
                }
            }
            DockNode::Container { children, .. } => {
                for child in children {
                    if Self::split_node_recursive(child, target_id, orientation, ratio, new_left, new_right)? {
                        return Ok(true);
                    }
                }
                Ok(false)
            }
            _ => Ok(false),
        }
    }
    
    /// Add a panel to a dock, creating tabs if needed
    pub fn add_panel(&mut self, dock_id: Uuid, tab_info: TabInfo) -> UiResult<()> {
        let mut root = std::mem::replace(&mut self.root, DockNode::Empty { config: DockConfig::default() });
        let result = Self::add_panel_recursive(&mut root, dock_id, tab_info);
        self.root = root;
        result?;
        self.mark_dirty();
        Ok(())
    }
    
    fn add_panel_recursive(node: &mut DockNode, dock_id: Uuid, tab_info: TabInfo) -> UiResult<bool> {
        match node {
            DockNode::Empty { config } if config.id == dock_id => {
                // Convert empty to tab container
                *node = DockNode::TabContainer {
                    config: config.clone(),
                    tabs: vec![tab_info],
                    active_tab: 0,
                };
                Ok(true)
            }
            DockNode::TabContainer { config, tabs, .. } if config.id == dock_id => {
                // Add to existing tab container
                tabs.push(tab_info);
                Ok(true)
            }
            DockNode::Container { children, .. } => {
                for child in children {
                    if Self::add_panel_recursive(child, dock_id, tab_info.clone())? {
                        return Ok(true);
                    }
                }
                Ok(false)
            }
            _ => Ok(false),
        }
    }
    
    /// Remove a panel from the docking system
    pub fn remove_panel(&mut self, panel_id: Uuid) -> UiResult<()> {
        let mut root = std::mem::replace(&mut self.root, DockNode::Empty { config: DockConfig::default() });
        let result = Self::remove_panel_recursive(&mut root, panel_id);
        self.root = root;
        result?;
        self.mark_dirty();
        Ok(())
    }
    
    fn remove_panel_recursive(node: &mut DockNode, panel_id: Uuid) -> UiResult<bool> {
        match node {
            DockNode::TabContainer { tabs, active_tab, config } => {
                if let Some(pos) = tabs.iter().position(|t| t.id == panel_id) {
                    tabs.remove(pos);
                    if tabs.is_empty() {
                        // Convert to empty if no tabs left
                        *node = DockNode::Empty { config: config.clone() };
                    } else if *active_tab >= tabs.len() {
                        *active_tab = tabs.len() - 1;
                    }
                    return Ok(true);
                }
                Ok(false)
            }
            DockNode::Container { children, .. } => {
                for child in children {
                    if Self::remove_panel_recursive(child, panel_id)? {
                        return Ok(true);
                    }
                }
                Ok(false)
            }
            _ => Ok(false),
        }
    }
    
    /// Start dragging a panel
    pub fn start_drag(&mut self, panel_id: Uuid, position: Vector2<f32>) -> UiResult<()> {
        // Find the dock containing this panel
        if let Some(dock_id) = self.find_panel_dock(&self.root, panel_id) {
            self.dragging = Some(DragState {
                panel_id,
                start_position: position,
                current_position: position,
                original_dock: dock_id,
            });
            Ok(())
        } else {
            Err(UiError::ElementNotFound(format!("Panel {} not found", panel_id)))
        }
    }
    
    fn find_panel_dock(&self, node: &DockNode, panel_id: Uuid) -> Option<Uuid> {
        match node {
            DockNode::TabContainer { config, tabs, .. } => {
                if tabs.iter().any(|t| t.id == panel_id) {
                    Some(config.id)
                } else {
                    None
                }
            }
            DockNode::Container { children, .. } => {
                for child in children {
                    if let Some(dock_id) = self.find_panel_dock(child, panel_id) {
                        return Some(dock_id);
                    }
                }
                None
            }
            _ => None,
        }
    }
    
    /// Update drag position and calculate drop preview
    pub fn update_drag(&mut self, position: Vector2<f32>) {
        if let Some(ref mut drag) = self.dragging {
            drag.current_position = position;
            
            // Calculate which dock we're hovering over
            self.hover_position = self.calculate_drop_position(position);
            self.mark_dirty();
        }
    }
    
    fn calculate_drop_position(&self, position: Vector2<f32>) -> Option<DockPosition> {
        // Find dock under cursor
        for (dock_id, bounds) in &self.panel_bounds {
            if bounds.contains(position) {
                // Calculate position within dock
                let relative = Vector2::new(
                    (position.x - bounds.position.x) / bounds.size.x,
                    (position.y - bounds.position.y) / bounds.size.y,
                );
                
                // Determine dock position based on relative position
                const EDGE_THRESHOLD: f32 = 0.2;
                
                if relative.x < EDGE_THRESHOLD {
                    return Some(DockPosition::Left);
                } else if relative.x > 1.0 - EDGE_THRESHOLD {
                    return Some(DockPosition::Right);
                } else if relative.y < EDGE_THRESHOLD {
                    return Some(DockPosition::Top);
                } else if relative.y > 1.0 - EDGE_THRESHOLD {
                    return Some(DockPosition::Bottom);
                } else {
                    return Some(DockPosition::Center);
                }
            }
        }
        None
    }
    
    /// Complete the drag operation
    pub fn end_drag(&mut self) -> UiResult<()> {
        if let Some(drag) = self.dragging.take() {
            if let Some(position) = self.hover_position.take() {
                // Find target dock
                let target_dock = self.panel_bounds.iter()
                    .find(|(_, bounds)| bounds.contains(drag.current_position))
                    .map(|(id, _)| *id);
                
                if let Some(dock_id) = target_dock {
                    // Remove from original location
                    let tab_info = self.extract_panel_info(drag.panel_id)?;
                    self.remove_panel(drag.panel_id)?;
                    
                    // Add to new location based on position
                    match position {
                        DockPosition::Center => {
                            // Add as tab
                            self.add_panel(dock_id, tab_info)?;
                        }
                        DockPosition::Left | DockPosition::Right => {
                            // Split horizontally
                            let ratio = if position == DockPosition::Left { 0.5 } else { 0.5 };
                            let (left, right) = self.split_dock(dock_id, DockOrientation::Horizontal, ratio)?;
                            let target = if position == DockPosition::Left { left } else { right };
                            self.add_panel(target, tab_info)?;
                        }
                        DockPosition::Top | DockPosition::Bottom => {
                            // Split vertically
                            let ratio = if position == DockPosition::Top { 0.5 } else { 0.5 };
                            let (top, bottom) = self.split_dock(dock_id, DockOrientation::Vertical, ratio)?;
                            let target = if position == DockPosition::Top { top } else { bottom };
                            self.add_panel(target, tab_info)?;
                        }
                    }
                }
            }
        }
        self.mark_dirty();
        Ok(())
    }
    
    fn extract_panel_info(&self, panel_id: Uuid) -> UiResult<TabInfo> {
        self.extract_panel_info_recursive(&self.root, panel_id)
    }
    
    fn extract_panel_info_recursive(&self, node: &DockNode, panel_id: Uuid) -> UiResult<TabInfo> {
        match node {
            DockNode::TabContainer { tabs, .. } => {
                if let Some(tab) = tabs.iter().find(|t| t.id == panel_id) {
                    Ok(tab.clone())
                } else {
                    Err(UiError::ElementNotFound(format!("Panel {} not found", panel_id)))
                }
            }
            DockNode::Container { children, .. } => {
                for child in children {
                    if let Ok(info) = self.extract_panel_info_recursive(child, panel_id) {
                        return Ok(info);
                    }
                }
                Err(UiError::ElementNotFound(format!("Panel {} not found", panel_id)))
            }
            _ => Err(UiError::ElementNotFound(format!("Panel {} not found", panel_id))),
        }
    }
    
    /// Calculate layout for all docks
    fn calculate_layout(&mut self) {
        self.panel_bounds.clear();
        self.resize_handles.clear();
        let root = self.root.clone();
        let bounds = self.bounds;
        self.calculate_node_layout(&root, bounds);
    }
    
    fn calculate_node_layout(&mut self, node: &DockNode, bounds: ElementBounds) {
        match node {
            DockNode::Container { config, orientation, children } => {
                self.panel_bounds.insert(config.id, bounds);
                
                let mut current_pos = bounds.position;
                
                // Calculate total size from children
                let mut child_configs = Vec::new();
                for child in children.iter() {
                    if let Some(cfg) = self.get_node_config(child) {
                        child_configs.push((child.clone(), cfg.id, cfg.size));
                    }
                }
                
                let total_size: f32 = child_configs.iter().map(|(_, _, size)| size).sum();
                
                for (i, (child, child_id, child_size)) in child_configs.iter().enumerate() {
                    let size_ratio = child_size / total_size;
                    
                    let child_bounds = match orientation {
                        DockOrientation::Horizontal => {
                            let width = bounds.size.x * size_ratio;
                            ElementBounds::new(
                                current_pos.x,
                                current_pos.y,
                                width - if i < children.len() - 1 { self.handle_width } else { 0.0 },
                                bounds.size.y,
                            )
                        }
                        DockOrientation::Vertical => {
                            let height = bounds.size.y * size_ratio;
                            ElementBounds::new(
                                current_pos.x,
                                current_pos.y,
                                bounds.size.x,
                                height - if i < children.len() - 1 { self.handle_width } else { 0.0 },
                            )
                        }
                    };
                    
                    self.calculate_node_layout(child, child_bounds);
                    
                    // Add resize handle between panels
                    if i < children.len() - 1 {
                        let handle_bounds = match orientation {
                            DockOrientation::Horizontal => {
                                current_pos.x += child_bounds.size.x;
                                let handle = ElementBounds::new(
                                    current_pos.x,
                                    current_pos.y,
                                    self.handle_width,
                                    bounds.size.y,
                                );
                                current_pos.x += self.handle_width;
                                handle
                            }
                            DockOrientation::Vertical => {
                                current_pos.y += child_bounds.size.y;
                                let handle = ElementBounds::new(
                                    current_pos.x,
                                    current_pos.y,
                                    bounds.size.x,
                                    self.handle_width,
                                );
                                current_pos.y += self.handle_width;
                                handle
                            }
                        };
                        
                        let right_id = if i + 1 < child_configs.len() {
                            child_configs[i + 1].1
                        } else {
                            Uuid::new_v4()
                        };
                        
                        self.resize_handles.push(ResizeHandle {
                            id: Uuid::new_v4(),
                            bounds: handle_bounds,
                            orientation: *orientation,
                            left_dock: *child_id,
                            right_dock: right_id,
                            hovering: false,
                            dragging: false,
                        });
                    } else {
                        match orientation {
                            DockOrientation::Horizontal => current_pos.x += child_bounds.size.x,
                            DockOrientation::Vertical => current_pos.y += child_bounds.size.y,
                        }
                    }
                }
            }
            DockNode::TabContainer { config, .. } | DockNode::Empty { config } => {
                self.panel_bounds.insert(config.id, bounds);
            }
        }
    }
    
    fn get_node_config<'a>(&self, node: &'a DockNode) -> Option<&'a DockConfig> {
        match node {
            DockNode::Container { config, .. } |
            DockNode::TabContainer { config, .. } |
            DockNode::Empty { config } => Some(config),
        }
    }
    
    /// Save layout to JSON
    pub fn save_layout(&self) -> UiResult<String> {
        let layout = SavedLayout {
            root: self.serialize_node(&self.root),
            bounds: self.bounds,
        };
        serde_json::to_string(&layout).map_err(|e| UiError::Other(format!("Failed to serialize layout: {}", e)))
    }
    
    fn serialize_node(&self, node: &DockNode) -> SavedNode {
        match node {
            DockNode::Container { config, orientation, children } => SavedNode::Container {
                config: config.clone(),
                orientation: *orientation,
                children: children.iter().map(|c| self.serialize_node(c)).collect(),
            },
            DockNode::TabContainer { config, tabs, active_tab } => SavedNode::TabContainer {
                config: config.clone(),
                tabs: tabs.clone(),
                active_tab: *active_tab,
            },
            DockNode::Empty { config } => SavedNode::Empty {
                config: config.clone(),
            },
        }
    }
    
    /// Load layout from JSON
    pub fn load_layout(&mut self, json: &str) -> UiResult<()> {
        let layout: SavedLayout = serde_json::from_str(json)
            .map_err(|e| UiError::Other(format!("Failed to deserialize layout: {}", e)))?;
        
        self.root = self.deserialize_node(&layout.root);
        self.bounds = layout.bounds;
        self.mark_dirty();
        Ok(())
    }
    
    fn deserialize_node(&self, node: &SavedNode) -> DockNode {
        match node {
            SavedNode::Container { config, orientation, children } => DockNode::Container {
                config: config.clone(),
                orientation: *orientation,
                children: children.iter().map(|c| self.deserialize_node(c)).collect(),
            },
            SavedNode::TabContainer { config, tabs, active_tab } => DockNode::TabContainer {
                config: config.clone(),
                tabs: tabs.clone(),
                active_tab: *active_tab,
            },
            SavedNode::Empty { config } => DockNode::Empty {
                config: config.clone(),
            },
        }
    }
}

/// Saved layout structure for serialization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SavedLayout {
    root: SavedNode,
    bounds: ElementBounds,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
enum SavedNode {
    Container {
        config: DockConfig,
        orientation: DockOrientation,
        children: Vec<SavedNode>,
    },
    TabContainer {
        config: DockConfig,
        tabs: Vec<TabInfo>,
        active_tab: usize,
    },
    Empty {
        config: DockConfig,
    },
}

impl Element for DockingLayout {
    fn id(&self) -> Uuid {
        self.id
    }
    
    fn type_name(&self) -> &str {
        "DockingLayout"
    }
    
    fn layout(&mut self, constraints: &LayoutConstraints) -> UiResult<LayoutResult> {
        // Use available size from constraints
        self.bounds = ElementBounds::new(
            0.0,
            0.0,
            constraints.available_size.x,
            constraints.available_size.y,
        );
        
        // Update orientation based on new bounds
        self.update_orientation(self.bounds.size.x, self.bounds.size.y);
        
        // Adjust handle and tab sizes for mobile
        if self.current_orientation == ScreenOrientation::Portrait {
            self.handle_width = 6.0 * self.responsive_config.mobile_size_multiplier;
            self.tab_height = 40.0 * self.responsive_config.mobile_size_multiplier;
        } else {
            self.handle_width = 4.0;
            self.tab_height = 30.0;
        }
        
        self.calculate_layout();
        
        Ok(LayoutResult::new(
            Vector2::new(self.bounds.size.x, self.bounds.size.y),
            Vector2::new(self.bounds.position.x, self.bounds.position.y),
        ))
    }
    
    fn handle_input(&mut self, event: &InputEvent) -> InputResult {
        match event {
            InputEvent::PointerDown { position, .. } => {
                // Check resize handles first
                for handle in &mut self.resize_handles {
                    if handle.bounds.contains(*position) {
                        handle.dragging = true;
                        return InputResult {
                            handled: EventHandled::Yes,
                            request_focus: true,
                        };
                    }
                }
                
                // Check for panel drag start (would need tab bar hit testing)
                InputResult {
                    handled: EventHandled::No,
                    request_focus: false,
                }
            }
            InputEvent::PointerMove { position, .. } => {
                // Update resize handle hover states
                for handle in &mut self.resize_handles {
                    handle.hovering = handle.bounds.contains(*position);
                }
                
                // Handle resize dragging
                if let Some(handle) = self.resize_handles.iter_mut().find(|h| h.dragging) {
                    // Update panel sizes based on drag
                    self.mark_dirty();
                    return InputResult {
                        handled: EventHandled::Yes,
                        request_focus: false,
                    };
                }
                
                // Handle panel dragging
                if self.dragging.is_some() {
                    self.update_drag(*position);
                    return InputResult {
                        handled: EventHandled::Yes,
                        request_focus: false,
                    };
                }
                
                InputResult {
                    handled: EventHandled::No,
                    request_focus: false,
                }
            }
            InputEvent::PointerUp { .. } => {
                // Release resize handles
                for handle in &mut self.resize_handles {
                    handle.dragging = false;
                }
                
                // Complete panel drag
                if self.dragging.is_some() {
                    let _ = self.end_drag();
                    return InputResult {
                        handled: EventHandled::Yes,
                        request_focus: false,
                    };
                }
                
                InputResult {
                    handled: EventHandled::No,
                    request_focus: false,
                }
            }
            _ => InputResult {
                handled: EventHandled::No,
                request_focus: false,
            },
        }
    }
    
    fn render(&self, theme: &Theme) -> UiResult<RenderData> {
        let mut vertices = Vec::new();
        let mut colors = Vec::new();
        let mut indices = Vec::new();
        
        // Render dock backgrounds
        self.render_node(&self.root, theme, &mut vertices, &mut colors, &mut indices);
        
        // Render resize handles
        for handle in &self.resize_handles {
            let color = if handle.dragging {
                theme.colors.primary
            } else if handle.hovering {
                theme.colors.hover
            } else {
                theme.colors.border
            };
            
            let base_idx = vertices.len() as u32;
            
            // Add vertices for handle rectangle
            vertices.push(Vector2::new(handle.bounds.position.x, handle.bounds.position.y));
            vertices.push(Vector2::new(handle.bounds.position.x + handle.bounds.size.x, handle.bounds.position.y));
            vertices.push(Vector2::new(handle.bounds.position.x + handle.bounds.size.x, handle.bounds.position.y + handle.bounds.size.y));
            vertices.push(Vector2::new(handle.bounds.position.x, handle.bounds.position.y + handle.bounds.size.y));
            
            for _ in 0..4 {
                colors.push(color);
            }
            
            indices.extend_from_slice(&[
                base_idx, base_idx + 1, base_idx + 2,
                base_idx, base_idx + 2, base_idx + 3,
            ]);
        }
        
        // Render drop preview if dragging
        if let Some(ref drag) = self.dragging {
            if let Some(_position) = self.hover_position {
                // Render semi-transparent preview overlay
                let preview_color = Vector4::new(
                    theme.colors.primary.x,
                    theme.colors.primary.y,
                    theme.colors.primary.z,
                    self.drop_preview_alpha,
                );
                
                // Calculate preview bounds based on drop position
                // (simplified - would need actual target dock bounds)
                let preview_bounds = ElementBounds::new(
                    drag.current_position.x - 50.0,
                    drag.current_position.y - 50.0,
                    100.0,
                    100.0,
                );
                
                let base_idx = vertices.len() as u32;
                
                vertices.push(Vector2::new(preview_bounds.position.x, preview_bounds.position.y));
                vertices.push(Vector2::new(preview_bounds.position.x + preview_bounds.size.x, preview_bounds.position.y));
                vertices.push(Vector2::new(preview_bounds.position.x + preview_bounds.size.x, preview_bounds.position.y + preview_bounds.size.y));
                vertices.push(Vector2::new(preview_bounds.position.x, preview_bounds.position.y + preview_bounds.size.y));
                
                for _ in 0..4 {
                    colors.push(preview_color);
                }
                
                indices.extend_from_slice(&[
                    base_idx, base_idx + 1, base_idx + 2,
                    base_idx, base_idx + 2, base_idx + 3,
                ]);
            }
        }
        
        Ok(RenderData {
            vertices,
            colors,
            indices,
            uvs: Vec::new(),
            texture_id: None,
            z_order: 0.0,
            scissor_rect: None,
        })
    }
    
    fn update(&mut self, _delta_time: f32) {
        // Update animations if needed
    }
    
    fn children(&self) -> &[ElementId] {
        &[]
    }
    
    fn children_mut(&mut self) -> &mut Vec<ElementId> {
        // Return a temporary empty vector - this is a placeholder implementation
        // In a real implementation, DockingLayout would have actual children storage
        Box::leak(Box::new(Vec::new()))
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
        self.bounds
    }
    
    fn set_bounds(&mut self, bounds: ElementBounds) {
        self.bounds = bounds;
        self.calculate_layout();
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

impl DockingLayout {
    fn render_node(
        &self,
        node: &DockNode,
        theme: &Theme,
        vertices: &mut Vec<Vector2<f32>>,
        colors: &mut Vec<Vector4<f32>>,
        indices: &mut Vec<u32>,
    ) {
        match node {
            DockNode::Container { children, .. } => {
                for child in children {
                    self.render_node(child, theme, vertices, colors, indices);
                }
            }
            DockNode::TabContainer { config, tabs, active_tab } => {
                if let Some(bounds) = self.panel_bounds.get(&config.id) {
                    // Render tab bar
                    let tab_bar_bounds = ElementBounds::new(
                        bounds.position.x,
                        bounds.position.y,
                        bounds.size.x,
                        self.tab_height,
                    );
                    
                    let base_idx = vertices.len() as u32;
                    
                    // Tab bar background
                    vertices.push(Vector2::new(tab_bar_bounds.position.x, tab_bar_bounds.position.y));
                    vertices.push(Vector2::new(tab_bar_bounds.position.x + tab_bar_bounds.size.x, tab_bar_bounds.position.y));
                    vertices.push(Vector2::new(tab_bar_bounds.position.x + tab_bar_bounds.size.x, tab_bar_bounds.position.y + tab_bar_bounds.size.y));
                    vertices.push(Vector2::new(tab_bar_bounds.position.x, tab_bar_bounds.position.y + tab_bar_bounds.size.y));
                    
                    for _ in 0..4 {
                        colors.push(theme.colors.surface);
                    }
                    
                    indices.extend_from_slice(&[
                        base_idx, base_idx + 1, base_idx + 2,
                        base_idx, base_idx + 2, base_idx + 3,
                    ]);
                    
                    // Render individual tabs
                    let tab_width = tab_bar_bounds.size.x / tabs.len() as f32;
                    for (i, tab) in tabs.iter().enumerate() {
                        let tab_x = tab_bar_bounds.position.x + (i as f32 * tab_width);
                        let tab_color = if i == *active_tab {
                            theme.colors.background
                        } else {
                            theme.colors.surface_variant
                        };
                        
                        let tab_base_idx = vertices.len() as u32;
                        
                        vertices.push(Vector2::new(tab_x, tab_bar_bounds.position.y));
                        vertices.push(Vector2::new(tab_x + tab_width - 1.0, tab_bar_bounds.position.y));
                        vertices.push(Vector2::new(tab_x + tab_width - 1.0, tab_bar_bounds.position.y + tab_bar_bounds.size.y));
                        vertices.push(Vector2::new(tab_x, tab_bar_bounds.position.y + tab_bar_bounds.size.y));
                        
                        for _ in 0..4 {
                            colors.push(tab_color);
                        }
                        
                        indices.extend_from_slice(&[
                            tab_base_idx, tab_base_idx + 1, tab_base_idx + 2,
                            tab_base_idx, tab_base_idx + 2, tab_base_idx + 3,
                        ]);
                    }
                    
                    // Render content area
                    let content_bounds = ElementBounds::new(
                        bounds.position.x,
                        bounds.position.y + self.tab_height,
                        bounds.size.x,
                        bounds.size.y - self.tab_height,
                    );
                    
                    let content_base_idx = vertices.len() as u32;
                    
                    vertices.push(Vector2::new(content_bounds.position.x, content_bounds.position.y));
                    vertices.push(Vector2::new(content_bounds.position.x + content_bounds.size.x, content_bounds.position.y));
                    vertices.push(Vector2::new(content_bounds.position.x + content_bounds.size.x, content_bounds.position.y + content_bounds.size.y));
                    vertices.push(Vector2::new(content_bounds.position.x, content_bounds.position.y + content_bounds.size.y));
                    
                    for _ in 0..4 {
                        colors.push(theme.colors.background);
                    }
                    
                    indices.extend_from_slice(&[
                        content_base_idx, content_base_idx + 1, content_base_idx + 2,
                        content_base_idx, content_base_idx + 2, content_base_idx + 3,
                    ]);
                }
            }
            DockNode::Empty { config } => {
                if let Some(bounds) = self.panel_bounds.get(&config.id) {
                    // Render empty dock area
                    let base_idx = vertices.len() as u32;
                    
                    vertices.push(Vector2::new(bounds.position.x, bounds.position.y));
                    vertices.push(Vector2::new(bounds.position.x + bounds.size.x, bounds.position.y));
                    vertices.push(Vector2::new(bounds.position.x + bounds.size.x, bounds.position.y + bounds.size.y));
                    vertices.push(Vector2::new(bounds.position.x, bounds.position.y + bounds.size.y));
                    
                    for _ in 0..4 {
                        colors.push(theme.colors.surface_variant);
                    }
                    
                    indices.extend_from_slice(&[
                        base_idx, base_idx + 1, base_idx + 2,
                        base_idx, base_idx + 2, base_idx + 3,
                    ]);
                }
            }
        }
    }
    
    fn update(&mut self, _delta_time: f32) {
        // Update animations if needed
    }
    
    fn children(&self) -> &[ElementId] {
        &[]
    }
    
    fn children_mut(&mut self) -> &mut Vec<ElementId> {
        // Return a temporary empty vector - this is a placeholder implementation
        // In a real implementation, DockingLayout would have actual children storage
        Box::leak(Box::new(Vec::new()))
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
        self.bounds
    }
    
    fn set_bounds(&mut self, bounds: ElementBounds) {
        self.bounds = bounds;
        self.calculate_layout();
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