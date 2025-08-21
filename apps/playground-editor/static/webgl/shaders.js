/**
 * WebGL2 Shader Management
 * Compiles and manages shader programs for 2D rendering
 */

export class ShaderManager {
    constructor(gl) {
        this.gl = gl;
        this.programs = new Map();
        this.currentProgram = null;
        
        // Initialize default shaders
        this.initializeDefaultShaders();
    }
    
    initializeDefaultShaders() {
        // Basic quad shader for rectangles and images
        this.createProgram('quad', 
            this.getQuadVertexShader(),
            this.getQuadFragmentShader()
        );
        
        // Line shader for borders and lines
        this.createProgram('line',
            this.getLineVertexShader(),
            this.getLineFragmentShader()
        );
        
        // Circle shader for rounded elements
        this.createProgram('circle',
            this.getCircleVertexShader(),
            this.getCircleFragmentShader()
        );
        
        // Text shader with SDF rendering
        this.createProgram('text',
            this.getTextVertexShader(),
            this.getTextFragmentShader()
        );
    }
    
    createProgram(name, vertexSource, fragmentSource) {
        const gl = this.gl;
        
        // Compile vertex shader
        const vertexShader = this.compileShader(gl.VERTEX_SHADER, vertexSource);
        if (!vertexShader) {
            console.error(`Failed to compile vertex shader for ${name}`);
            return null;
        }
        
        // Compile fragment shader
        const fragmentShader = this.compileShader(gl.FRAGMENT_SHADER, fragmentSource);
        if (!fragmentShader) {
            console.error(`Failed to compile fragment shader for ${name}`);
            gl.deleteShader(vertexShader);
            return null;
        }
        
        // Create and link program
        const program = gl.createProgram();
        gl.attachShader(program, vertexShader);
        gl.attachShader(program, fragmentShader);
        gl.linkProgram(program);
        
        if (!gl.getProgramParameter(program, gl.LINK_STATUS)) {
            console.error(`Failed to link program ${name}: ${gl.getProgramInfoLog(program)}`);
            gl.deleteProgram(program);
            gl.deleteShader(vertexShader);
            gl.deleteShader(fragmentShader);
            return null;
        }
        
        // Clean up shaders (they're part of the program now)
        gl.deleteShader(vertexShader);
        gl.deleteShader(fragmentShader);
        
        // Get attribute and uniform locations
        const attributes = this.getAttributeLocations(program);
        const uniforms = this.getUniformLocations(program);
        
        // Store program info
        this.programs.set(name, {
            program,
            attributes,
            uniforms
        });
        
        return program;
    }
    
    compileShader(type, source) {
        const gl = this.gl;
        const shader = gl.createShader(type);
        
        gl.shaderSource(shader, source);
        gl.compileShader(shader);
        
        if (!gl.getShaderParameter(shader, gl.COMPILE_STATUS)) {
            console.error(`Shader compilation error: ${gl.getShaderInfoLog(shader)}`);
            gl.deleteShader(shader);
            return null;
        }
        
        return shader;
    }
    
    getAttributeLocations(program) {
        const gl = this.gl;
        const count = gl.getProgramParameter(program, gl.ACTIVE_ATTRIBUTES);
        const attributes = {};
        
        for (let i = 0; i < count; i++) {
            const info = gl.getActiveAttrib(program, i);
            attributes[info.name] = gl.getAttribLocation(program, info.name);
        }
        
        return attributes;
    }
    
    getUniformLocations(program) {
        const gl = this.gl;
        const count = gl.getProgramParameter(program, gl.ACTIVE_UNIFORMS);
        const uniforms = {};
        
        for (let i = 0; i < count; i++) {
            const info = gl.getActiveUniform(program, i);
            uniforms[info.name] = gl.getUniformLocation(program, info.name);
        }
        
        return uniforms;
    }
    
    useProgram(name) {
        const programInfo = this.programs.get(name);
        if (!programInfo) {
            console.error(`Program ${name} not found`);
            return null;
        }
        
        if (this.currentProgram !== programInfo.program) {
            this.gl.useProgram(programInfo.program);
            this.currentProgram = programInfo.program;
        }
        
        return programInfo;
    }
    
    // Shader sources
    
    getQuadVertexShader() {
        return `#version 300 es
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
        }`;
    }
    
    getQuadFragmentShader() {
        return `#version 300 es
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
        }`;
    }
    
    getLineVertexShader() {
        return `#version 300 es
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
        }`;
    }
    
    getLineFragmentShader() {
        return `#version 300 es
        precision highp float;
        
        in vec4 v_color;
        out vec4 fragColor;
        
        void main() {
            fragColor = v_color;
        }`;
    }
    
    getCircleVertexShader() {
        return `#version 300 es
        precision highp float;
        
        in vec2 a_position;
        in vec2 a_center;
        in float a_radius;
        in vec4 a_color;
        
        uniform mat3 u_projection;
        uniform mat3 u_transform;
        
        out vec2 v_position;
        out vec2 v_center;
        out float v_radius;
        out vec4 v_color;
        
        void main() {
            vec3 position = u_projection * u_transform * vec3(a_position, 1.0);
            gl_Position = vec4(position.xy, 0.0, 1.0);
            v_position = a_position;
            v_center = a_center;
            v_radius = a_radius;
            v_color = a_color;
        }`;
    }
    
    getCircleFragmentShader() {
        return `#version 300 es
        precision highp float;
        
        in vec2 v_position;
        in vec2 v_center;
        in float v_radius;
        in vec4 v_color;
        
        uniform bool u_filled;
        
        out vec4 fragColor;
        
        void main() {
            float dist = distance(v_position, v_center);
            
            if (u_filled) {
                if (dist <= v_radius) {
                    fragColor = v_color;
                } else {
                    discard;
                }
            } else {
                float thickness = 2.0;
                if (abs(dist - v_radius) <= thickness) {
                    fragColor = v_color;
                } else {
                    discard;
                }
            }
        }`;
    }
    
    getTextVertexShader() {
        return `#version 300 es
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
        }`;
    }
    
    getTextFragmentShader() {
        return `#version 300 es
        precision highp float;
        
        in vec2 v_texCoord;
        in vec4 v_color;
        
        uniform sampler2D u_texture;
        
        out vec4 fragColor;
        
        void main() {
            float alpha = texture(u_texture, v_texCoord).a;
            fragColor = vec4(v_color.rgb, v_color.a * alpha);
        }`;
    }
    
    dispose() {
        const gl = this.gl;
        
        for (const [name, info] of this.programs) {
            gl.deleteProgram(info.program);
        }
        
        this.programs.clear();
        this.currentProgram = null;
    }
}