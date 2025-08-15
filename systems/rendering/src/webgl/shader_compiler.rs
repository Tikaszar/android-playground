use web_sys::{WebGl2RenderingContext, WebGlShader, WebGlProgram};
use std::collections::HashMap;
use crate::error::RendererError;
use crate::resources::ShaderStage;

pub struct ShaderCompiler {
    shader_cache: HashMap<String, WebGlShader>,
    program_cache: HashMap<(String, String), WebGlProgram>,
}

impl ShaderCompiler {
    pub fn new() -> Self {
        Self {
            shader_cache: HashMap::new(),
            program_cache: HashMap::new(),
        }
    }

    pub fn compile_shader(
        &mut self,
        gl: &WebGl2RenderingContext,
        source: &str,
        stage: ShaderStage,
    ) -> Result<WebGlShader, RendererError> {
        // Check cache first
        if let Some(shader) = self.shader_cache.get(source) {
            log::debug!("Using cached shader");
            return Ok(shader.clone());
        }

        let shader_type = match stage {
            ShaderStage::Vertex => WebGl2RenderingContext::VERTEX_SHADER,
            ShaderStage::Fragment => WebGl2RenderingContext::FRAGMENT_SHADER,
            _ => return Err(RendererError::NotSupported(format!("Shader stage {:?} not supported in WebGL", stage))),
        };

        let shader = gl.create_shader(shader_type)
            .ok_or_else(|| RendererError::ShaderCompilationFailed("Failed to create shader".to_string()))?;

        // Add WebGL-specific boilerplate
        let full_source = format!("#version 300 es\nprecision highp float;\n{}", source);
        
        gl.shader_source(&shader, &full_source);
        gl.compile_shader(&shader);

        if !gl.get_shader_parameter(&shader, WebGl2RenderingContext::COMPILE_STATUS)
            .as_bool()
            .unwrap_or(false)
        {
            let error = gl.get_shader_info_log(&shader)
                .unwrap_or_else(|| "Unknown shader compilation error".to_string());
            gl.delete_shader(Some(&shader));
            return Err(RendererError::ShaderCompilationFailed(error));
        }

        self.shader_cache.insert(source.to_string(), shader.clone());
        log::info!("Shader compiled successfully");
        Ok(shader)
    }

    pub fn link_program(
        &mut self,
        gl: &WebGl2RenderingContext,
        vertex_shader: &WebGlShader,
        fragment_shader: &WebGlShader,
    ) -> Result<WebGlProgram, RendererError> {
        let program = gl.create_program()
            .ok_or_else(|| RendererError::PipelineCreationFailed("Failed to create program".to_string()))?;

        gl.attach_shader(&program, vertex_shader);
        gl.attach_shader(&program, fragment_shader);
        gl.link_program(&program);

        if !gl.get_program_parameter(&program, WebGl2RenderingContext::LINK_STATUS)
            .as_bool()
            .unwrap_or(false)
        {
            let error = gl.get_program_info_log(&program)
                .unwrap_or_else(|| "Unknown program linking error".to_string());
            gl.delete_program(Some(&program));
            return Err(RendererError::PipelineCreationFailed(error));
        }

        log::info!("Shader program linked successfully");
        Ok(program)
    }

    pub fn clear_cache(&mut self) {
        self.shader_cache.clear();
        self.program_cache.clear();
    }
}