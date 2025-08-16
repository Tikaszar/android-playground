use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use nalgebra::{Vector2, Vector3, Vector4};
use crate::error::{UiError, UiResult};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct GlyphId(pub u32);

#[derive(Debug, Clone, Copy)]
pub struct GlyphMetrics {
    pub advance: f32,
    pub bearing_x: f32,
    pub bearing_y: f32,
    pub width: f32,
    pub height: f32,
}

#[derive(Debug, Clone)]
pub struct GlyphInfo {
    pub id: GlyphId,
    pub codepoint: char,
    pub metrics: GlyphMetrics,
    pub atlas_x: u32,
    pub atlas_y: u32,
    pub atlas_width: u32,
    pub atlas_height: u32,
}

#[derive(Debug, Clone)]
pub struct FontAtlas {
    pub texture_data: Vec<u8>,
    pub texture_width: u32,
    pub texture_height: u32,
    pub glyphs: HashMap<char, GlyphInfo>,
    pub font_size: f32,
    pub sdf_radius: f32,
}

impl FontAtlas {
    pub fn new(width: u32, height: u32, font_size: f32, sdf_radius: f32) -> Self {
        Self {
            texture_data: vec![0; (width * height * 4) as usize],
            texture_width: width,
            texture_height: height,
            glyphs: HashMap::new(),
            font_size,
            sdf_radius,
        }
    }

    pub fn add_glyph(&mut self, codepoint: char, info: GlyphInfo) {
        self.glyphs.insert(codepoint, info);
    }

    pub fn get_glyph(&self, codepoint: char) -> Option<&GlyphInfo> {
        self.glyphs.get(&codepoint)
    }

    pub fn pack_glyph_data(&mut self, x: u32, y: u32, width: u32, height: u32, data: &[u8]) -> UiResult<()> {
        if x + width > self.texture_width || y + height > self.texture_height {
            return Err(UiError::InvalidInput("Glyph exceeds atlas bounds".into()));
        }

        for row in 0..height {
            for col in 0..width {
                let atlas_idx = ((y + row) * self.texture_width + (x + col)) * 4;
                let data_idx = (row * width + col) as usize;
                
                if atlas_idx as usize + 3 < self.texture_data.len() && data_idx < data.len() {
                    let value = data[data_idx];
                    self.texture_data[atlas_idx as usize] = value;
                    self.texture_data[atlas_idx as usize + 1] = value;
                    self.texture_data[atlas_idx as usize + 2] = value;
                    self.texture_data[atlas_idx as usize + 3] = value;
                }
            }
        }

        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct Font {
    pub name: String,
    pub atlas: Arc<RwLock<FontAtlas>>,
    pub fallback_font: Option<Arc<Font>>,
}

impl Font {
    pub fn new(name: String, atlas: FontAtlas) -> Self {
        Self {
            name,
            atlas: Arc::new(RwLock::new(atlas)),
            fallback_font: None,
        }
    }

    pub fn with_fallback(mut self, fallback: Arc<Font>) -> Self {
        self.fallback_font = Some(fallback);
        self
    }

    pub fn get_glyph(&self, codepoint: char) -> Option<GlyphInfo> {
        if let Ok(atlas) = self.atlas.read() {
            if let Some(glyph) = atlas.get_glyph(codepoint) {
                return Some(glyph.clone());
            }
        }

        if let Some(fallback) = &self.fallback_font {
            return fallback.get_glyph(codepoint);
        }

        None
    }
}

#[derive(Debug, Clone, Copy)]
pub struct TextMetrics {
    pub width: f32,
    pub height: f32,
    pub line_height: f32,
    pub ascent: f32,
    pub descent: f32,
}

#[derive(Debug, Clone)]
pub struct TextLayout {
    pub lines: Vec<TextLine>,
    pub metrics: TextMetrics,
}

#[derive(Debug, Clone)]
pub struct TextLine {
    pub glyphs: Vec<PositionedGlyph>,
    pub width: f32,
    pub height: f32,
    pub baseline_y: f32,
}

#[derive(Debug, Clone)]
pub struct PositionedGlyph {
    pub glyph: GlyphInfo,
    pub position: Vector2<f32>,
    pub color: Vector4<f32>,
}

pub struct TextRenderer {
    fonts: HashMap<String, Arc<Font>>,
    default_font: Option<Arc<Font>>,
    cache: Arc<RwLock<HashMap<String, TextLayout>>>,
}

impl TextRenderer {
    pub fn new() -> Self {
        Self {
            fonts: HashMap::new(),
            default_font: None,
            cache: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub fn add_font(&mut self, font: Font) -> Arc<Font> {
        let font_arc = Arc::new(font);
        self.fonts.insert(font_arc.name.clone(), font_arc.clone());
        
        if self.default_font.is_none() {
            self.default_font = Some(font_arc.clone());
        }
        
        font_arc
    }

    pub fn set_default_font(&mut self, font_name: &str) -> UiResult<()> {
        if let Some(font) = self.fonts.get(font_name) {
            self.default_font = Some(font.clone());
            Ok(())
        } else {
            Err(UiError::NotFound(format!("Font '{}' not found", font_name)))
        }
    }

    pub fn layout_text(
        &self,
        text: &str,
        font_name: Option<&str>,
        font_size: f32,
        max_width: Option<f32>,
    ) -> UiResult<TextLayout> {
        let cache_key = format!("{}:{}:{}:{:?}", text, font_name.unwrap_or("default"), font_size, max_width);
        
        if let Ok(cache) = self.cache.read() {
            if let Some(layout) = cache.get(&cache_key) {
                return Ok(layout.clone());
            }
        }

        let font = if let Some(name) = font_name {
            self.fonts.get(name)
                .ok_or_else(|| UiError::NotFound(format!("Font '{}' not found", name)))?
        } else {
            self.default_font.as_ref()
                .ok_or_else(|| UiError::NotFound("No default font set".into()))?
        };

        let layout = self.perform_layout(text, font, font_size, max_width)?;

        if let Ok(mut cache) = self.cache.write() {
            cache.insert(cache_key, layout.clone());
        }

        Ok(layout)
    }

    fn perform_layout(
        &self,
        text: &str,
        font: &Arc<Font>,
        font_size: f32,
        max_width: Option<f32>,
    ) -> UiResult<TextLayout> {
        let mut lines = Vec::new();
        let mut current_line = TextLine {
            glyphs: Vec::new(),
            width: 0.0,
            height: font_size,
            baseline_y: 0.0,
        };

        let atlas = font.atlas.read()
            .map_err(|_| UiError::LockError("Failed to acquire font atlas lock".into()))?;
        let scale = font_size / atlas.font_size;

        let mut cursor_x = 0.0;
        let mut cursor_y = 0.0;
        let line_height = font_size * 1.2;

        for ch in text.chars() {
            if ch == '\n' {
                current_line.baseline_y = cursor_y + font_size * 0.8;
                lines.push(current_line);
                current_line = TextLine {
                    glyphs: Vec::new(),
                    width: 0.0,
                    height: font_size,
                    baseline_y: 0.0,
                };
                cursor_x = 0.0;
                cursor_y += line_height;
                continue;
            }

            if ch == ' ' {
                cursor_x += font_size * 0.25;
                current_line.width = cursor_x;
                continue;
            }

            if let Some(glyph) = font.get_glyph(ch) {
                let glyph_width = glyph.metrics.advance * scale;

                if let Some(max_w) = max_width {
                    if cursor_x + glyph_width > max_w && cursor_x > 0.0 {
                        current_line.baseline_y = cursor_y + font_size * 0.8;
                        lines.push(current_line);
                        current_line = TextLine {
                            glyphs: Vec::new(),
                            width: 0.0,
                            height: font_size,
                            baseline_y: 0.0,
                        };
                        cursor_x = 0.0;
                        cursor_y += line_height;
                    }
                }

                let positioned = PositionedGlyph {
                    glyph: glyph.clone(),
                    position: Vector2::new(cursor_x, cursor_y),
                    color: Vector4::new(1.0, 1.0, 1.0, 1.0),
                };

                current_line.glyphs.push(positioned);
                cursor_x += glyph_width;
                current_line.width = cursor_x;
            }
        }

        if !current_line.glyphs.is_empty() || lines.is_empty() {
            current_line.baseline_y = cursor_y + font_size * 0.8;
            lines.push(current_line);
        }

        let total_width = lines.iter().map(|l| l.width).fold(0.0f32, f32::max);
        let total_height = if lines.is_empty() {
            0.0
        } else {
            cursor_y + line_height
        };

        Ok(TextLayout {
            lines,
            metrics: TextMetrics {
                width: total_width,
                height: total_height,
                line_height,
                ascent: font_size * 0.8,
                descent: font_size * 0.2,
            },
        })
    }

    pub fn measure_text(
        &self,
        text: &str,
        font_name: Option<&str>,
        font_size: f32,
    ) -> UiResult<TextMetrics> {
        let layout = self.layout_text(text, font_name, font_size, None)?;
        Ok(layout.metrics)
    }

    pub fn clear_cache(&self) {
        if let Ok(mut cache) = self.cache.write() {
            cache.clear();
        }
    }
}

pub fn generate_sdf_from_bitmap(
    bitmap: &[u8],
    width: u32,
    height: u32,
    spread: f32,
) -> Vec<u8> {
    let mut sdf = vec![0u8; (width * height) as usize];
    let spread_sq = spread * spread;

    for y in 0..height {
        for x in 0..width {
            let idx = (y * width + x) as usize;
            let inside = bitmap[idx] > 127;
            
            let mut min_dist_sq = spread_sq;

            for dy in -(spread as i32)..=(spread as i32) {
                for dx in -(spread as i32)..=(spread as i32) {
                    let nx = x as i32 + dx;
                    let ny = y as i32 + dy;

                    if nx >= 0 && nx < width as i32 && ny >= 0 && ny < height as i32 {
                        let nidx = (ny as u32 * width + nx as u32) as usize;
                        let neighbor_inside = bitmap[nidx] > 127;

                        if inside != neighbor_inside {
                            let dist_sq = (dx * dx + dy * dy) as f32;
                            min_dist_sq = min_dist_sq.min(dist_sq);
                        }
                    }
                }
            }

            let dist = min_dist_sq.sqrt();
            let normalized = if inside {
                0.5 + 0.5 * (dist / spread).min(1.0)
            } else {
                0.5 - 0.5 * (dist / spread).min(1.0)
            };

            sdf[idx] = (normalized * 255.0) as u8;
        }
    }

    sdf
}

#[derive(Debug, Clone)]
pub struct TextVertex {
    pub position: Vector3<f32>,
    pub tex_coord: Vector2<f32>,
    pub color: Vector4<f32>,
}

pub fn generate_text_mesh(layout: &TextLayout, origin: Vector2<f32>) -> Vec<TextVertex> {
    let mut vertices = Vec::new();

    for line in &layout.lines {
        for glyph in &line.glyphs {
            let atlas = glyph.glyph.atlas_width as f32;
            let atlas_height = glyph.glyph.atlas_height as f32;

            let x = origin.x + glyph.position.x + glyph.glyph.metrics.bearing_x;
            let y = origin.y + glyph.position.y - glyph.glyph.metrics.bearing_y;
            let w = glyph.glyph.metrics.width;
            let h = glyph.glyph.metrics.height;

            let u0 = glyph.glyph.atlas_x as f32 / atlas;
            let v0 = glyph.glyph.atlas_y as f32 / atlas_height;
            let u1 = (glyph.glyph.atlas_x + glyph.glyph.atlas_width) as f32 / atlas;
            let v1 = (glyph.glyph.atlas_y + glyph.glyph.atlas_height) as f32 / atlas_height;

            vertices.push(TextVertex {
                position: Vector3::new(x, y, 0.0),
                tex_coord: Vector2::new(u0, v0),
                color: glyph.color,
            });

            vertices.push(TextVertex {
                position: Vector3::new(x + w, y, 0.0),
                tex_coord: Vector2::new(u1, v0),
                color: glyph.color,
            });

            vertices.push(TextVertex {
                position: Vector3::new(x + w, y + h, 0.0),
                tex_coord: Vector2::new(u1, v1),
                color: glyph.color,
            });

            vertices.push(TextVertex {
                position: Vector3::new(x, y, 0.0),
                tex_coord: Vector2::new(u0, v0),
                color: glyph.color,
            });

            vertices.push(TextVertex {
                position: Vector3::new(x + w, y + h, 0.0),
                tex_coord: Vector2::new(u1, v1),
                color: glyph.color,
            });

            vertices.push(TextVertex {
                position: Vector3::new(x, y + h, 0.0),
                tex_coord: Vector2::new(u0, v1),
                color: glyph.color,
            });
        }
    }

    vertices
}