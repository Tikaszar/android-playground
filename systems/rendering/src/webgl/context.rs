use web_sys::{HtmlCanvasElement, WebGl2RenderingContext};
use wasm_bindgen::JsCast;
use crate::error::RendererError;

pub struct WebGLContext {
    gl: WebGl2RenderingContext,
    canvas: HtmlCanvasElement,
}

impl WebGLContext {
    pub fn new(canvas: HtmlCanvasElement) -> Result<Self, RendererError> {
        let gl = canvas
            .get_context("webgl2")
            .map_err(|e| RendererError::InitializationFailed(format!("Failed to get WebGL2 context: {:?}", e)))?
            .ok_or_else(|| RendererError::InitializationFailed("WebGL2 context is null".to_string()))?
            .dyn_into::<WebGl2RenderingContext>()
            .map_err(|_| RendererError::InitializationFailed("Failed to cast to WebGL2RenderingContext".to_string()))?;

        log::info!("WebGL2 context created successfully");

        Ok(Self { gl, canvas })
    }

    pub fn gl(&self) -> &WebGl2RenderingContext {
        &self.gl
    }

    pub fn canvas(&self) -> &HtmlCanvasElement {
        &self.canvas
    }

    pub fn resize(&self, width: u32, height: u32) {
        self.canvas.set_width(width);
        self.canvas.set_height(height);
        self.gl.viewport(0, 0, width as i32, height as i32);
        log::debug!("Canvas resized to {}x{}", width, height);
    }

    pub fn get_error(&self) -> Option<String> {
        match self.gl.get_error() {
            WebGl2RenderingContext::NO_ERROR => None,
            WebGl2RenderingContext::INVALID_ENUM => Some("INVALID_ENUM".to_string()),
            WebGl2RenderingContext::INVALID_VALUE => Some("INVALID_VALUE".to_string()),
            WebGl2RenderingContext::INVALID_OPERATION => Some("INVALID_OPERATION".to_string()),
            WebGl2RenderingContext::INVALID_FRAMEBUFFER_OPERATION => Some("INVALID_FRAMEBUFFER_OPERATION".to_string()),
            WebGl2RenderingContext::OUT_OF_MEMORY => Some("OUT_OF_MEMORY".to_string()),
            WebGl2RenderingContext::CONTEXT_LOST_WEBGL => Some("CONTEXT_LOST_WEBGL".to_string()),
            _ => Some("Unknown error".to_string()),
        }
    }

    pub fn check_error(&self, operation: &str) -> Result<(), RendererError> {
        if let Some(error) = self.get_error() {
            Err(RendererError::WebGLError(format!("{}: {}", operation, error)))
        } else {
            Ok(())
        }
    }
}