/**
 * WebGL2 Text Rendering
 * Handles text measurement and rendering using canvas-based texture atlases
 */

export class TextRenderer {
    constructor(gl, textureManager) {
        this.gl = gl;
        this.textureManager = textureManager;
        
        // Text rendering canvas
        this.canvas = document.createElement('canvas');
        this.ctx = this.canvas.getContext('2d', { alpha: true });
        
        // Text cache for performance
        this.textCache = new Map();
        this.cacheSize = 0;
        this.maxCacheSize = 100;
        
        // Font settings
        this.defaultFont = 'system-ui, -apple-system, sans-serif';
        this.fontCache = new Map();
        
        // Texture atlas for text (not implemented in this simplified version)
        this.textTextureId = 1000; // Reserved texture ID for text
    }
    
    measureText(text, fontSize, fontFamily = null) {
        const font = `${fontSize}px ${fontFamily || this.defaultFont}`;
        
        // Check cache
        const cacheKey = `${text}|${font}`;
        if (this.fontCache.has(cacheKey)) {
            return this.fontCache.get(cacheKey);
        }
        
        // Measure text
        this.ctx.font = font;
        const metrics = this.ctx.measureText(text);
        
        // Calculate dimensions
        const width = Math.ceil(metrics.width);
        const height = Math.ceil(fontSize * 1.4); // Approximate height
        
        const result = {
            width,
            height,
            ascent: metrics.actualBoundingBoxAscent || fontSize * 0.8,
            descent: metrics.actualBoundingBoxDescent || fontSize * 0.2
        };
        
        // Cache result
        this.fontCache.set(cacheKey, result);
        
        return result;
    }
    
    renderText(text, position, fontSize, color) {
        if (!text || text.length === 0) return;
        
        // Create cache key
        const cacheKey = `${text}|${fontSize}|${color.join(',')}`;
        
        // Check if we have cached texture
        let textureData = this.textCache.get(cacheKey);
        
        if (!textureData) {
            // Render text to canvas
            textureData = this.renderTextToCanvas(text, fontSize, color);
            
            // Add to cache
            this.addToCache(cacheKey, textureData);
        }
        
        // Create texture from canvas if needed
        if (!textureData.textureId) {
            textureData.textureId = this.textTextureId++;
            this.textureManager.createTextureFromCanvas(
                textureData.textureId,
                textureData.canvas
            );
        }
        
        // Get texture
        const texture = this.textureManager.get(textureData.textureId);
        
        // Bind texture and render as quad
        this.gl.activeTexture(this.gl.TEXTURE0);
        this.gl.bindTexture(this.gl.TEXTURE_2D, texture);
        
        // Return texture info for renderer to use
        return {
            texture,
            width: textureData.width,
            height: textureData.height,
            position
        };
    }
    
    renderTextToCanvas(text, fontSize, color) {
        // Set up font
        const font = `${fontSize}px ${this.defaultFont}`;
        this.ctx.font = font;
        
        // Measure text
        const metrics = this.ctx.measureText(text);
        const width = Math.ceil(metrics.width) + 4; // Add padding
        const height = Math.ceil(fontSize * 1.4) + 4;
        
        // Resize canvas if needed
        if (this.canvas.width < width || this.canvas.height < height) {
            this.canvas.width = Math.max(this.canvas.width, width);
            this.canvas.height = Math.max(this.canvas.height, height);
        }
        
        // Clear canvas
        this.ctx.clearRect(0, 0, width, height);
        
        // Set text properties
        this.ctx.font = font;
        this.ctx.textAlign = 'left';
        this.ctx.textBaseline = 'top';
        
        // For WebGL, we need to render white text and use color in shader
        // This allows for better batching
        this.ctx.fillStyle = `rgba(${Math.floor(color[0] * 255)}, ${Math.floor(color[1] * 255)}, ${Math.floor(color[2] * 255)}, ${color[3]})`;
        
        // Draw text
        this.ctx.fillText(text, 2, 2 + fontSize * 0.2); // Offset for padding and baseline
        
        // Create a new canvas for this text (to avoid conflicts)
        const textCanvas = document.createElement('canvas');
        textCanvas.width = width;
        textCanvas.height = height;
        const textCtx = textCanvas.getContext('2d');
        textCtx.drawImage(this.canvas, 0, 0, width, height, 0, 0, width, height);
        
        return {
            canvas: textCanvas,
            width,
            height,
            textureId: null
        };
    }
    
    addToCache(key, data) {
        // Remove oldest entries if cache is full
        if (this.cacheSize >= this.maxCacheSize) {
            const firstKey = this.textCache.keys().next().value;
            const firstData = this.textCache.get(firstKey);
            
            // Delete texture if it exists
            if (firstData && firstData.textureId) {
                this.textureManager.deleteTexture(firstData.textureId);
            }
            
            this.textCache.delete(firstKey);
            this.cacheSize--;
        }
        
        // Add new entry
        this.textCache.set(key, data);
        this.cacheSize++;
    }
    
    clearCache() {
        // Delete all cached textures
        for (const [key, data] of this.textCache) {
            if (data.textureId) {
                this.textureManager.deleteTexture(data.textureId);
            }
        }
        
        this.textCache.clear();
        this.fontCache.clear();
        this.cacheSize = 0;
    }
    
    renderTextBatch(texts) {
        // Batch rendering optimization for multiple text elements
        // This would ideally use a texture atlas, but for simplicity
        // we'll just ensure textures are created efficiently
        
        const results = [];
        
        for (const { text, position, fontSize, color } of texts) {
            const result = this.renderText(text, position, fontSize, color);
            if (result) {
                results.push(result);
            }
        }
        
        return results;
    }
    
    setDefaultFont(fontFamily) {
        this.defaultFont = fontFamily;
        this.clearCache(); // Clear cache when font changes
    }
    
    dispose() {
        this.clearCache();
        this.canvas = null;
        this.ctx = null;
    }
}