/**
 * WebGL2 Renderer
 * Main renderer that executes RenderCommand batches from the server
 */

import { WebGLContextManager } from './context.js';
import { ShaderManager } from './shaders.js';
import { BufferManager } from './buffers.js';
import { TextureManager } from './textures.js';
import { TextRenderer } from './text.js';

export class WebGLRenderer {
    constructor(canvas) {
        this.canvas = canvas;
        this.context = new WebGLContextManager(canvas);
        this.gl = this.context.gl;
        
        this.shaders = new ShaderManager(this.gl);
        this.buffers = new BufferManager(this.gl);
        this.textures = new TextureManager(this.gl);
        this.textRenderer = new TextRenderer(this.gl, this.textures);
        
        // Transform and state stacks
        this.transformStack = [this.createIdentityMatrix()];
        this.clipRectStack = [];
        this.stateStack = [];
        
        // Current render state
        this.currentTransform = this.createIdentityMatrix();
        this.currentClipRect = null;
        this.currentOpacity = 1.0;
        
        // Stats
        this.frameCount = 0;
        this.drawCalls = 0;
    }
    
    createIdentityMatrix() {
        return new Float32Array([
            1, 0, 0,
            0, 1, 0,
            0, 0, 1
        ]);
    }
    
    createTranslationMatrix(x, y) {
        return new Float32Array([
            1, 0, 0,
            0, 1, 0,
            x, y, 1
        ]);
    }
    
    createScaleMatrix(sx, sy) {
        return new Float32Array([
            sx, 0, 0,
            0, sy, 0,
            0, 0, 1
        ]);
    }
    
    createRotationMatrix(angle) {
        const cos = Math.cos(angle);
        const sin = Math.sin(angle);
        return new Float32Array([
            cos, sin, 0,
            -sin, cos, 0,
            0, 0, 1
        ]);
    }
    
    multiplyMatrices(a, b) {
        const result = new Float32Array(9);
        
        for (let i = 0; i < 3; i++) {
            for (let j = 0; j < 3; j++) {
                let sum = 0;
                for (let k = 0; k < 3; k++) {
                    sum += a[i * 3 + k] * b[k * 3 + j];
                }
                result[i * 3 + j] = sum;
            }
        }
        
        return result;
    }
    
    executeCommandBatch(batch) {
        if (!this.context.isReady()) return;
        
        // Reset stats
        this.drawCalls = 0;
        
        // Clear if needed
        if (batch.clear) {
            this.context.clear();
        }
        
        // Get projection matrix for current viewport
        const projection = this.context.getProjectionMatrix();
        
        // Use quad shader program for drawing
        const program = this.shaders.useProgram('quad');
        if (!program) {
            console.error('Quad shader program not found');
            return;
        }
        
        // Set projection and transform uniforms
        this.gl.uniformMatrix3fv(program.uniforms.u_projection, false, projection);
        this.gl.uniformMatrix3fv(program.uniforms.u_transform, false, this.currentTransform);
        this.gl.uniform1i(program.uniforms.u_useTexture, 0); // No texture by default
        
        // Execute each command
        for (const command of batch.commands) {
            this.executeCommand(command, projection);
        }
        
        // Flush any remaining batched geometry
        this.buffers.flush();
        
        this.frameCount++;
    }
    
    isInitialized() {
        return this.context && this.context.isReady() && this.shaders && this.buffers;
    }
    
    executeCommand(command, projection) {
        // Handle Clear command
        if (command.Clear) {
            const { color } = command.Clear;
            this.context.setClearColor(color[0], color[1], color[2], color[3]);
            this.context.clear();
            this.drawCalls++;
            return;
        }
        
        // Handle DrawQuad command
        if (command.DrawQuad) {
            const { position, size, color } = command.DrawQuad;
            this.drawQuad(position, size, color, projection);
            return;
        }
        
        // Handle DrawText command
        if (command.DrawText) {
            const { text, position, size, color } = command.DrawText;
            this.drawText(text, position, size, color, projection);
            return;
        }
        
        // Handle DrawImage command
        if (command.DrawImage) {
            const { texture_id, position, size, uv_min, uv_max } = command.DrawImage;
            this.drawImage(texture_id, position, size, uv_min, uv_max, projection);
            return;
        }
        
        // Handle DrawLine command
        if (command.DrawLine) {
            const { start, end, width, color } = command.DrawLine;
            this.drawLine(start, end, width, color, projection);
            return;
        }
        
        // Handle DrawCircle command
        if (command.DrawCircle) {
            const { center, radius, color, filled } = command.DrawCircle;
            this.drawCircle(center, radius, color, filled, projection);
            return;
        }
        
        // Handle SetClipRect command
        if (command.SetClipRect) {
            const { position, size } = command.SetClipRect;
            this.setClipRect(position, size);
            return;
        }
        
        // Handle ClearClipRect command
        if (command.ClearClipRect) {
            this.clearClipRect();
            return;
        }
        
        // Handle SetTransform command
        if (command.SetTransform) {
            const { matrix } = command.SetTransform;
            this.setTransform(matrix);
            return;
        }
        
        // Handle ResetTransform command
        if (command.ResetTransform) {
            this.resetTransform();
            return;
        }
        
        // Handle PushState command
        if (command.PushState) {
            this.pushState();
            return;
        }
        
        // Handle PopState command
        if (command.PopState) {
            this.popState();
            return;
        }
    }
    
    drawQuad(position, size, color, projection) {
        // Apply opacity
        const finalColor = [
            color[0],
            color[1],
            color[2],
            color[3] * this.currentOpacity
        ];
        
        // Add quad to buffer
        this.buffers.addQuad(
            position[0], position[1],
            size[0], size[1],
            finalColor
        );
        
        this.drawCalls++;
    }
    
    drawText(text, position, size, color, projection) {
        // Flush existing geometry first
        this.buffers.flush();
        
        // Use quad shader for text
        const program = this.shaders.useProgram('text');
        if (!program) return;
        
        // Set uniforms
        this.gl.uniformMatrix3fv(program.uniforms.u_projection, false, projection);
        this.gl.uniformMatrix3fv(program.uniforms.u_transform, false, this.currentTransform);
        
        // Apply opacity
        const finalColor = [
            color[0],
            color[1],
            color[2],
            color[3] * this.currentOpacity
        ];
        
        // Render text using text renderer
        this.textRenderer.renderText(text, position, size, finalColor);
        
        this.drawCalls++;
    }
    
    drawImage(textureId, position, size, uvMin, uvMax, projection) {
        // Get or load texture
        const texture = this.textures.get(textureId);
        if (!texture) {
            console.warn(`Texture ${textureId} not found`);
            return;
        }
        
        // Flush existing geometry first
        this.buffers.flush();
        
        // Use quad shader
        const program = this.shaders.useProgram('quad');
        if (!program) return;
        
        // Set uniforms
        this.gl.uniformMatrix3fv(program.uniforms.u_projection, false, projection);
        this.gl.uniformMatrix3fv(program.uniforms.u_transform, false, this.currentTransform);
        this.gl.uniform1i(program.uniforms.u_useTexture, 1);
        
        // Bind texture
        this.gl.activeTexture(this.gl.TEXTURE0);
        this.gl.bindTexture(this.gl.TEXTURE_2D, texture);
        this.gl.uniform1i(program.uniforms.u_texture, 0);
        
        // Add textured quad
        const texCoords = [uvMin[0], uvMin[1], uvMax[0], uvMax[1]];
        this.buffers.addQuad(
            position[0], position[1],
            size[0], size[1],
            [1, 1, 1, this.currentOpacity],
            texCoords
        );
        
        // Draw immediately for texture changes
        this.buffers.flush();
        this.drawCalls++;
    }
    
    drawLine(start, end, width, color, projection) {
        // Apply opacity
        const finalColor = [
            color[0],
            color[1],
            color[2],
            color[3] * this.currentOpacity
        ];
        
        // For thick lines, draw as a quad
        if (width > 1) {
            const dx = end[0] - start[0];
            const dy = end[1] - start[1];
            const len = Math.sqrt(dx * dx + dy * dy);
            
            if (len > 0) {
                const nx = -dy / len * width * 0.5;
                const ny = dx / len * width * 0.5;
                
                // Create quad vertices for thick line
                // This is a simplified approach - proper would use line shader
                this.buffers.addQuad(
                    Math.min(start[0], end[0]) - Math.abs(nx),
                    Math.min(start[1], end[1]) - Math.abs(ny),
                    Math.abs(dx) + Math.abs(nx) * 2,
                    Math.abs(dy) + Math.abs(ny) * 2,
                    finalColor
                );
            }
        } else {
            // Add thin line
            this.buffers.addLine(start[0], start[1], end[0], end[1], finalColor);
        }
    }
    
    drawCircle(center, radius, color, filled, projection) {
        // Approximate circle with a polygon for now
        const segments = Math.max(16, Math.floor(radius / 2));
        const angleStep = (Math.PI * 2) / segments;
        
        const finalColor = [
            color[0],
            color[1],
            color[2],
            color[3] * this.currentOpacity
        ];
        
        if (filled) {
            // Draw as triangle fan (simplified)
            for (let i = 0; i < segments; i++) {
                const angle1 = i * angleStep;
                const angle2 = (i + 1) * angleStep;
                
                const x1 = center[0] + Math.cos(angle1) * radius;
                const y1 = center[1] + Math.sin(angle1) * radius;
                const x2 = center[0] + Math.cos(angle2) * radius;
                const y2 = center[1] + Math.sin(angle2) * radius;
                
                // Draw triangle from center to edge
                // Simplified - would be better with proper circle shader
                this.buffers.addQuad(
                    Math.min(center[0], x1, x2),
                    Math.min(center[1], y1, y2),
                    Math.abs(Math.max(x1, x2) - Math.min(center[0], x1, x2)),
                    Math.abs(Math.max(y1, y2) - Math.min(center[1], y1, y2)),
                    finalColor
                );
            }
        } else {
            // Draw circle outline
            for (let i = 0; i < segments; i++) {
                const angle1 = i * angleStep;
                const angle2 = (i + 1) * angleStep;
                
                const x1 = center[0] + Math.cos(angle1) * radius;
                const y1 = center[1] + Math.sin(angle1) * radius;
                const x2 = center[0] + Math.cos(angle2) * radius;
                const y2 = center[1] + Math.sin(angle2) * radius;
                
                this.buffers.addLine(x1, y1, x2, y2, finalColor);
            }
        }
    }
    
    setClipRect(position, size) {
        this.currentClipRect = { 
            x: position[0], 
            y: position[1], 
            width: size[0], 
            height: size[1] 
        };
        this.context.setScissor(position[0], position[1], size[0], size[1]);
    }
    
    clearClipRect() {
        this.currentClipRect = null;
        this.context.resetScissor();
    }
    
    setTransform(matrix) {
        this.currentTransform = new Float32Array(matrix);
    }
    
    resetTransform() {
        this.currentTransform = this.createIdentityMatrix();
    }
    
    pushState() {
        this.stateStack.push({
            transform: new Float32Array(this.currentTransform),
            clipRect: this.currentClipRect ? { ...this.currentClipRect } : null,
            opacity: this.currentOpacity
        });
    }
    
    popState() {
        const state = this.stateStack.pop();
        if (state) {
            this.currentTransform = state.transform;
            this.currentClipRect = state.clipRect;
            this.currentOpacity = state.opacity;
            
            if (state.clipRect) {
                this.context.setScissor(
                    state.clipRect.x,
                    state.clipRect.y,
                    state.clipRect.width,
                    state.clipRect.height
                );
            } else {
                this.context.resetScissor();
            }
        }
    }
    
    resize() {
        this.context.updateViewport();
    }
    
    dispose() {
        this.buffers.dispose();
        this.shaders.dispose();
        this.textures.dispose();
        this.textRenderer.dispose();
        this.context.dispose();
    }
}