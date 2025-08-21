/**
 * WebGL2 Texture Management
 * Handles texture loading, caching, and binding
 */

export class TextureManager {
    constructor(gl) {
        this.gl = gl;
        this.textures = new Map();
        this.pendingLoads = new Map();
        this.whitePixelTexture = null;
        
        this.initialize();
    }
    
    initialize() {
        // Create a white pixel texture for untextured rendering
        this.whitePixelTexture = this.createWhitePixelTexture();
        this.textures.set(0, this.whitePixelTexture);
    }
    
    createWhitePixelTexture() {
        const gl = this.gl;
        const texture = gl.createTexture();
        
        gl.bindTexture(gl.TEXTURE_2D, texture);
        gl.texImage2D(
            gl.TEXTURE_2D,
            0,
            gl.RGBA,
            1, 1, 0,
            gl.RGBA,
            gl.UNSIGNED_BYTE,
            new Uint8Array([255, 255, 255, 255])
        );
        
        gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_MIN_FILTER, gl.NEAREST);
        gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_MAG_FILTER, gl.NEAREST);
        gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_WRAP_S, gl.CLAMP_TO_EDGE);
        gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_WRAP_T, gl.CLAMP_TO_EDGE);
        
        return texture;
    }
    
    createTexture(id, width, height, data = null, options = {}) {
        const gl = this.gl;
        
        // Delete existing texture if it exists
        if (this.textures.has(id)) {
            this.deleteTexture(id);
        }
        
        const texture = gl.createTexture();
        gl.bindTexture(gl.TEXTURE_2D, texture);
        
        // Set texture parameters
        const {
            format = gl.RGBA,
            internalFormat = gl.RGBA,
            type = gl.UNSIGNED_BYTE,
            minFilter = gl.LINEAR,
            magFilter = gl.LINEAR,
            wrapS = gl.CLAMP_TO_EDGE,
            wrapT = gl.CLAMP_TO_EDGE,
            generateMipmap = false
        } = options;
        
        // Upload texture data
        if (data) {
            gl.texImage2D(
                gl.TEXTURE_2D,
                0,
                internalFormat,
                width, height, 0,
                format,
                type,
                data
            );
        } else {
            // Create empty texture
            gl.texImage2D(
                gl.TEXTURE_2D,
                0,
                internalFormat,
                width, height, 0,
                format,
                type,
                null
            );
        }
        
        // Set texture parameters
        gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_MIN_FILTER, minFilter);
        gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_MAG_FILTER, magFilter);
        gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_WRAP_S, wrapS);
        gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_WRAP_T, wrapT);
        
        // Generate mipmaps if requested
        if (generateMipmap && minFilter !== gl.NEAREST && minFilter !== gl.LINEAR) {
            gl.generateMipmap(gl.TEXTURE_2D);
        }
        
        // Store texture
        this.textures.set(id, texture);
        
        return texture;
    }
    
    async loadImage(id, url) {
        // Check if already loading
        if (this.pendingLoads.has(id)) {
            return this.pendingLoads.get(id);
        }
        
        // Check if already loaded
        if (this.textures.has(id)) {
            return this.textures.get(id);
        }
        
        // Create loading promise
        const loadPromise = new Promise((resolve, reject) => {
            const image = new Image();
            image.crossOrigin = 'anonymous';
            
            image.onload = () => {
                const texture = this.createTextureFromImage(id, image);
                this.pendingLoads.delete(id);
                resolve(texture);
            };
            
            image.onerror = (error) => {
                console.error(`Failed to load image ${url}:`, error);
                this.pendingLoads.delete(id);
                reject(error);
            };
            
            image.src = url;
        });
        
        this.pendingLoads.set(id, loadPromise);
        return loadPromise;
    }
    
    createTextureFromImage(id, image) {
        const gl = this.gl;
        
        const texture = gl.createTexture();
        gl.bindTexture(gl.TEXTURE_2D, texture);
        
        // Upload image to texture
        gl.texImage2D(
            gl.TEXTURE_2D,
            0,
            gl.RGBA,
            gl.RGBA,
            gl.UNSIGNED_BYTE,
            image
        );
        
        // Set texture parameters
        gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_MIN_FILTER, gl.LINEAR);
        gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_MAG_FILTER, gl.LINEAR);
        gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_WRAP_S, gl.CLAMP_TO_EDGE);
        gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_WRAP_T, gl.CLAMP_TO_EDGE);
        
        // Store texture
        this.textures.set(id, texture);
        
        return texture;
    }
    
    createTextureFromCanvas(id, canvas) {
        const gl = this.gl;
        
        const texture = gl.createTexture();
        gl.bindTexture(gl.TEXTURE_2D, texture);
        
        // Upload canvas to texture
        gl.texImage2D(
            gl.TEXTURE_2D,
            0,
            gl.RGBA,
            gl.RGBA,
            gl.UNSIGNED_BYTE,
            canvas
        );
        
        // Set texture parameters for text (no filtering for sharp text)
        gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_MIN_FILTER, gl.LINEAR);
        gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_MAG_FILTER, gl.LINEAR);
        gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_WRAP_S, gl.CLAMP_TO_EDGE);
        gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_WRAP_T, gl.CLAMP_TO_EDGE);
        
        // Store texture
        this.textures.set(id, texture);
        
        return texture;
    }
    
    updateTexture(id, x, y, width, height, data) {
        const texture = this.textures.get(id);
        if (!texture) {
            console.warn(`Texture ${id} not found for update`);
            return;
        }
        
        const gl = this.gl;
        gl.bindTexture(gl.TEXTURE_2D, texture);
        gl.texSubImage2D(
            gl.TEXTURE_2D,
            0,
            x, y,
            width, height,
            gl.RGBA,
            gl.UNSIGNED_BYTE,
            data
        );
    }
    
    get(id) {
        return this.textures.get(id) || this.whitePixelTexture;
    }
    
    has(id) {
        return this.textures.has(id);
    }
    
    deleteTexture(id) {
        const texture = this.textures.get(id);
        if (texture && texture !== this.whitePixelTexture) {
            this.gl.deleteTexture(texture);
            this.textures.delete(id);
        }
    }
    
    bind(texture, unit = 0) {
        const gl = this.gl;
        gl.activeTexture(gl.TEXTURE0 + unit);
        gl.bindTexture(gl.TEXTURE_2D, texture);
    }
    
    unbind(unit = 0) {
        const gl = this.gl;
        gl.activeTexture(gl.TEXTURE0 + unit);
        gl.bindTexture(gl.TEXTURE_2D, null);
    }
    
    dispose() {
        const gl = this.gl;
        
        for (const [id, texture] of this.textures) {
            gl.deleteTexture(texture);
        }
        
        this.textures.clear();
        this.pendingLoads.clear();
        this.whitePixelTexture = null;
    }
}