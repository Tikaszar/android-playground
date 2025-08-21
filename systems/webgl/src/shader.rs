use std::collections::HashMap;

pub enum ShaderType {
    Vertex,
    Fragment,
    Compute,
}

pub struct ShaderProgram {
    program_id: u32,
    uniform_locations: HashMap<String, i32>,
}

impl ShaderProgram {
    pub fn new(vertex_source: &str, fragment_source: &str) -> Result<Self, String> {
        Ok(Self {
            program_id: 0,
            uniform_locations: HashMap::new(),
        })
    }

    pub fn compile(&mut self) -> Result<(), String> {
        Ok(())
    }

    pub fn bind(&self) {
    }

    pub fn unbind(&self) {
    }

    pub fn get_uniform_location(&mut self, name: &str) -> Option<i32> {
        if let Some(&location) = self.uniform_locations.get(name) {
            return Some(location);
        }
        
        let location = 0;
        self.uniform_locations.insert(name.to_string(), location);
        Some(location)
    }

    pub fn set_uniform_matrix4(&self, name: &str, matrix: &[f32; 16]) {
    }

    pub fn set_uniform_vec4(&self, name: &str, values: &[f32; 4]) {
    }

    pub fn set_uniform_float(&self, name: &str, value: f32) {
    }

    pub fn set_uniform_int(&self, name: &str, value: i32) {
    }

    pub fn set_uniform_texture(&self, name: &str, texture_unit: u32) {
    }
}

pub const DEFAULT_VERTEX_SHADER: &str = r#"
#version 300 es
precision highp float;

layout(location = 0) in vec2 a_position;
layout(location = 1) in vec2 a_texcoord;
layout(location = 2) in vec4 a_color;

uniform mat4 u_projection;
uniform mat4 u_view;
uniform mat4 u_model;

out vec2 v_texcoord;
out vec4 v_color;

void main() {
    gl_Position = u_projection * u_view * u_model * vec4(a_position, 0.0, 1.0);
    v_texcoord = a_texcoord;
    v_color = a_color;
}
"#;

pub const DEFAULT_FRAGMENT_SHADER: &str = r#"
#version 300 es
precision highp float;

in vec2 v_texcoord;
in vec4 v_color;

uniform sampler2D u_texture;
uniform float u_use_texture;

out vec4 fragColor;

void main() {
    vec4 texColor = texture(u_texture, v_texcoord);
    fragColor = mix(v_color, texColor * v_color, u_use_texture);
}
"#;