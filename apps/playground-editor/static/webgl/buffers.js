/**
 * WebGL2 Buffer Management
 * Handles vertex and index buffer creation, updates, and batching
 */

export class BufferManager {
    constructor(gl) {
        this.gl = gl;
        
        // Vertex buffer for quads (position, texCoord, color)
        this.quadVertexBuffer = null;
        this.quadVertexData = null;
        this.quadVertexCount = 0;
        this.maxQuadVertices = 65536;
        
        // Index buffer for quads
        this.quadIndexBuffer = null;
        this.quadIndexData = null;
        this.quadIndexCount = 0;
        
        // Line vertex buffer
        this.lineVertexBuffer = null;
        this.lineVertexData = null;
        this.lineVertexCount = 0;
        this.maxLineVertices = 16384;
        
        // Vertex Array Objects
        this.quadVAO = null;
        this.lineVAO = null;
        
        this.initialize();
    }
    
    initialize() {
        this.createQuadBuffers();
        this.createLineBuffers();
    }
    
    createQuadBuffers() {
        const gl = this.gl;
        
        // Create vertex buffer
        this.quadVertexBuffer = gl.createBuffer();
        this.quadVertexData = new Float32Array(this.maxQuadVertices * 8); // 2 pos + 2 tex + 4 color
        
        // Create index buffer with pre-calculated indices
        this.quadIndexBuffer = gl.createBuffer();
        const indices = new Uint16Array(this.maxQuadVertices * 6 / 4);
        
        // Pre-calculate quad indices (0,1,2, 2,3,0 pattern)
        let offset = 0;
        for (let i = 0; i < indices.length; i += 6) {
            indices[i] = offset;
            indices[i + 1] = offset + 1;
            indices[i + 2] = offset + 2;
            indices[i + 3] = offset + 2;
            indices[i + 4] = offset + 3;
            indices[i + 5] = offset;
            offset += 4;
        }
        
        // Upload index data (static, won't change)
        gl.bindBuffer(gl.ELEMENT_ARRAY_BUFFER, this.quadIndexBuffer);
        gl.bufferData(gl.ELEMENT_ARRAY_BUFFER, indices, gl.STATIC_DRAW);
        
        // Create VAO for quads
        this.quadVAO = gl.createVertexArray();
        gl.bindVertexArray(this.quadVAO);
        
        // Bind vertex buffer
        gl.bindBuffer(gl.ARRAY_BUFFER, this.quadVertexBuffer);
        gl.bufferData(gl.ARRAY_BUFFER, this.quadVertexData.byteLength, gl.DYNAMIC_DRAW);
        
        // Set up vertex attributes (assuming shader locations)
        const stride = 32; // 8 floats * 4 bytes
        gl.enableVertexAttribArray(0); // position
        gl.vertexAttribPointer(0, 2, gl.FLOAT, false, stride, 0);
        
        gl.enableVertexAttribArray(1); // texCoord
        gl.vertexAttribPointer(1, 2, gl.FLOAT, false, stride, 8);
        
        gl.enableVertexAttribArray(2); // color
        gl.vertexAttribPointer(2, 4, gl.FLOAT, false, stride, 16);
        
        // Bind index buffer to VAO
        gl.bindBuffer(gl.ELEMENT_ARRAY_BUFFER, this.quadIndexBuffer);
        
        // Unbind VAO
        gl.bindVertexArray(null);
    }
    
    createLineBuffers() {
        const gl = this.gl;
        
        // Create vertex buffer for lines
        this.lineVertexBuffer = gl.createBuffer();
        this.lineVertexData = new Float32Array(this.maxLineVertices * 6); // 2 pos + 4 color
        
        // Create VAO for lines
        this.lineVAO = gl.createVertexArray();
        gl.bindVertexArray(this.lineVAO);
        
        // Bind vertex buffer
        gl.bindBuffer(gl.ARRAY_BUFFER, this.lineVertexBuffer);
        gl.bufferData(gl.ARRAY_BUFFER, this.lineVertexData.byteLength, gl.DYNAMIC_DRAW);
        
        // Set up vertex attributes
        const stride = 24; // 6 floats * 4 bytes
        gl.enableVertexAttribArray(0); // position
        gl.vertexAttribPointer(0, 2, gl.FLOAT, false, stride, 0);
        
        gl.enableVertexAttribArray(2); // color
        gl.vertexAttribPointer(2, 4, gl.FLOAT, false, stride, 8);
        
        // Unbind VAO
        gl.bindVertexArray(null);
    }
    
    addQuad(x, y, width, height, color, texCoords = null) {
        if (this.quadVertexCount + 4 > this.maxQuadVertices) {
            this.flushQuads();
        }
        
        const idx = this.quadVertexCount * 8;
        const data = this.quadVertexData;
        
        // Default texture coordinates
        const u0 = texCoords ? texCoords[0] : 0;
        const v0 = texCoords ? texCoords[1] : 0;
        const u1 = texCoords ? texCoords[2] : 1;
        const v1 = texCoords ? texCoords[3] : 1;
        
        // Top-left vertex
        data[idx] = x;
        data[idx + 1] = y;
        data[idx + 2] = u0;
        data[idx + 3] = v0;
        data[idx + 4] = color[0];
        data[idx + 5] = color[1];
        data[idx + 6] = color[2];
        data[idx + 7] = color[3];
        
        // Top-right vertex
        data[idx + 8] = x + width;
        data[idx + 9] = y;
        data[idx + 10] = u1;
        data[idx + 11] = v0;
        data[idx + 12] = color[0];
        data[idx + 13] = color[1];
        data[idx + 14] = color[2];
        data[idx + 15] = color[3];
        
        // Bottom-right vertex
        data[idx + 16] = x + width;
        data[idx + 17] = y + height;
        data[idx + 18] = u1;
        data[idx + 19] = v1;
        data[idx + 20] = color[0];
        data[idx + 21] = color[1];
        data[idx + 22] = color[2];
        data[idx + 23] = color[3];
        
        // Bottom-left vertex
        data[idx + 24] = x;
        data[idx + 25] = y + height;
        data[idx + 26] = u0;
        data[idx + 27] = v1;
        data[idx + 28] = color[0];
        data[idx + 29] = color[1];
        data[idx + 30] = color[2];
        data[idx + 31] = color[3];
        
        this.quadVertexCount += 4;
        this.quadIndexCount += 6;
    }
    
    addLine(x0, y0, x1, y1, color) {
        if (this.lineVertexCount + 2 > this.maxLineVertices) {
            this.flushLines();
        }
        
        const idx = this.lineVertexCount * 6;
        const data = this.lineVertexData;
        
        // Start vertex
        data[idx] = x0;
        data[idx + 1] = y0;
        data[idx + 2] = color[0];
        data[idx + 3] = color[1];
        data[idx + 4] = color[2];
        data[idx + 5] = color[3];
        
        // End vertex
        data[idx + 6] = x1;
        data[idx + 7] = y1;
        data[idx + 8] = color[0];
        data[idx + 9] = color[1];
        data[idx + 10] = color[2];
        data[idx + 11] = color[3];
        
        this.lineVertexCount += 2;
    }
    
    flushQuads() {
        if (this.quadVertexCount === 0) return;
        
        const gl = this.gl;
        
        // Upload vertex data
        gl.bindBuffer(gl.ARRAY_BUFFER, this.quadVertexBuffer);
        gl.bufferSubData(gl.ARRAY_BUFFER, 0, 
            this.quadVertexData.subarray(0, this.quadVertexCount * 8));
        
        // Bind VAO and draw
        gl.bindVertexArray(this.quadVAO);
        gl.drawElements(gl.TRIANGLES, this.quadIndexCount, gl.UNSIGNED_SHORT, 0);
        gl.bindVertexArray(null);
        
        // Reset counters
        this.quadVertexCount = 0;
        this.quadIndexCount = 0;
    }
    
    flushLines() {
        if (this.lineVertexCount === 0) return;
        
        const gl = this.gl;
        
        // Upload vertex data
        gl.bindBuffer(gl.ARRAY_BUFFER, this.lineVertexBuffer);
        gl.bufferSubData(gl.ARRAY_BUFFER, 0,
            this.lineVertexData.subarray(0, this.lineVertexCount * 6));
        
        // Bind VAO and draw
        gl.bindVertexArray(this.lineVAO);
        gl.drawArrays(gl.LINES, 0, this.lineVertexCount);
        gl.bindVertexArray(null);
        
        // Reset counter
        this.lineVertexCount = 0;
    }
    
    flush() {
        this.flushQuads();
        this.flushLines();
    }
    
    dispose() {
        const gl = this.gl;
        
        if (this.quadVertexBuffer) gl.deleteBuffer(this.quadVertexBuffer);
        if (this.quadIndexBuffer) gl.deleteBuffer(this.quadIndexBuffer);
        if (this.lineVertexBuffer) gl.deleteBuffer(this.lineVertexBuffer);
        if (this.quadVAO) gl.deleteVertexArray(this.quadVAO);
        if (this.lineVAO) gl.deleteVertexArray(this.lineVAO);
        
        this.quadVertexBuffer = null;
        this.quadIndexBuffer = null;
        this.lineVertexBuffer = null;
        this.quadVAO = null;
        this.lineVAO = null;
    }
}