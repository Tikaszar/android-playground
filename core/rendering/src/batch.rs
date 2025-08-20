use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RenderCommand {
    Clear {
        color: [f32; 4],
    },
    DrawQuad {
        position: [f32; 2],
        size: [f32; 2],
        color: [f32; 4],
    },
    DrawText {
        text: String,
        position: [f32; 2],
        size: f32,
        color: [f32; 4],
    },
    DrawImage {
        texture_id: u32,
        position: [f32; 2],
        size: [f32; 2],
        uv_min: [f32; 2],
        uv_max: [f32; 2],
    },
    DrawLine {
        start: [f32; 2],
        end: [f32; 2],
        width: f32,
        color: [f32; 4],
    },
    DrawCircle {
        center: [f32; 2],
        radius: f32,
        color: [f32; 4],
        filled: bool,
    },
    SetClipRect {
        position: [f32; 2],
        size: [f32; 2],
    },
    ClearClipRect,
    SetTransform {
        matrix: [[f32; 3]; 3],
    },
    ResetTransform,
    PushState,
    PopState,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RenderCommandBatch {
    commands: Vec<RenderCommand>,
    viewport: Option<Viewport>,
    frame_id: u64,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Viewport {
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
}

impl RenderCommandBatch {
    pub fn new(frame_id: u64) -> Self {
        Self {
            commands: Vec::new(),
            viewport: None,
            frame_id,
        }
    }
    
    pub fn with_capacity(capacity: usize, frame_id: u64) -> Self {
        Self {
            commands: Vec::with_capacity(capacity),
            viewport: None,
            frame_id,
        }
    }
    
    pub fn push(&mut self, command: RenderCommand) {
        self.commands.push(command);
    }
    
    pub fn extend(&mut self, commands: impl IntoIterator<Item = RenderCommand>) {
        self.commands.extend(commands);
    }
    
    pub fn clear(&mut self) {
        self.commands.clear();
    }
    
    pub fn set_viewport(&mut self, viewport: Viewport) {
        self.viewport = Some(viewport);
    }
    
    pub fn commands(&self) -> &[RenderCommand] {
        &self.commands
    }
    
    pub fn take_commands(&mut self) -> Vec<RenderCommand> {
        std::mem::take(&mut self.commands)
    }
    
    pub fn viewport(&self) -> Option<Viewport> {
        self.viewport
    }
    
    pub fn frame_id(&self) -> u64 {
        self.frame_id
    }
    
    pub fn is_empty(&self) -> bool {
        self.commands.is_empty()
    }
    
    pub fn len(&self) -> usize {
        self.commands.len()
    }
}