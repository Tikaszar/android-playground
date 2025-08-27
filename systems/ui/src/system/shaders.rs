use crate::system::UiSystem;

impl UiSystem {
    /// Get default quad vertex shader source
    pub fn get_quad_vertex_shader(&self) -> String {
        r#"#version 300 es
        precision highp float;
        
        in vec2 a_position;
        in vec2 a_texCoord;
        in vec4 a_color;
        
        uniform mat3 u_projection;
        uniform mat3 u_transform;
        
        out vec2 v_texCoord;
        out vec4 v_color;
        
        void main() {
            vec3 position = u_projection * u_transform * vec3(a_position, 1.0);
            gl_Position = vec4(position.xy, 0.0, 1.0);
            v_texCoord = a_texCoord;
            v_color = a_color;
        }"#.to_string()
    }
    
    /// Get default quad fragment shader source
    pub fn get_quad_fragment_shader(&self) -> String {
        r#"#version 300 es
        precision highp float;
        
        in vec2 v_texCoord;
        in vec4 v_color;
        
        uniform sampler2D u_texture;
        uniform bool u_useTexture;
        
        out vec4 fragColor;
        
        void main() {
            if (u_useTexture) {
                fragColor = texture(u_texture, v_texCoord) * v_color;
            } else {
                fragColor = v_color;
            }
        }"#.to_string()
    }
    
    /// Get default line vertex shader source
    pub fn get_line_vertex_shader(&self) -> String {
        r#"#version 300 es
        precision highp float;
        
        in vec2 a_position;
        in vec4 a_color;
        
        uniform mat3 u_projection;
        uniform mat3 u_transform;
        
        out vec4 v_color;
        
        void main() {
            vec3 position = u_projection * u_transform * vec3(a_position, 1.0);
            gl_Position = vec4(position.xy, 0.0, 1.0);
            v_color = a_color;
        }"#.to_string()
    }
    
    /// Get default line fragment shader source
    pub fn get_line_fragment_shader(&self) -> String {
        r#"#version 300 es
        precision highp float;
        
        in vec4 v_color;
        out vec4 fragColor;
        
        void main() {
            fragColor = v_color;
        }"#.to_string()
    }
    
    /// Get default text vertex shader source
    pub fn get_text_vertex_shader(&self) -> String {
        r#"#version 300 es
        precision highp float;
        
        in vec2 a_position;
        in vec2 a_texCoord;
        in vec4 a_color;
        
        uniform mat3 u_projection;
        uniform mat3 u_transform;
        
        out vec2 v_texCoord;
        out vec4 v_color;
        
        void main() {
            vec3 position = u_projection * u_transform * vec3(a_position, 1.0);
            gl_Position = vec4(position.xy, 0.0, 1.0);
            v_texCoord = a_texCoord;
            v_color = a_color;
        }"#.to_string()
    }
    
    /// Get default text fragment shader source  
    pub fn get_text_fragment_shader(&self) -> String {
        r#"#version 300 es
        precision highp float;
        
        in vec2 v_texCoord;
        in vec4 v_color;
        
        uniform sampler2D u_texture;
        
        out vec4 fragColor;
        
        void main() {
            float alpha = texture(u_texture, v_texCoord).a;
            fragColor = vec4(v_color.rgb, v_color.a * alpha);
        }"#.to_string()
    }
}