/**
 * Resource Cache for WebGL Renderer
 * Manages cached shaders, textures, and other resources
 */

export class ResourceCache {
    constructor() {
        this.shaders = new Map();
        this.textures = new Map();
        this.metadata = new Map();
        this.maxCacheSize = 100 * 1024 * 1024; // 100MB max cache
        this.currentSize = 0;
    }
    
    /**
     * Cache a shader program
     */
    cacheShader(id, program, source) {
        if (this.shaders.has(id)) {
            console.log(`Shader ${id} already cached, updating`);
        }
        
        this.shaders.set(id, {
            program: program,
            source: source,
            lastUsed: Date.now(),
            useCount: 0
        });
        
        // Estimate size (rough approximation)
        const size = JSON.stringify(source).length;
        this.currentSize += size;
        this.metadata.set(`shader:${id}`, { size, type: 'shader' });
        
        this.evictIfNeeded();
    }
    
    /**
     * Get a cached shader
     */
    getShader(id) {
        const shader = this.shaders.get(id);
        if (shader) {
            shader.lastUsed = Date.now();
            shader.useCount++;
            return shader.program;
        }
        return null;
    }
    
    /**
     * Cache a texture
     */
    cacheTexture(id, texture, metadata) {
        if (this.textures.has(id)) {
            console.log(`Texture ${id} already cached, updating`);
            this.removeTexture(id);
        }
        
        this.textures.set(id, {
            texture: texture,
            metadata: metadata,
            lastUsed: Date.now(),
            useCount: 0
        });
        
        // Calculate texture size
        const size = (metadata.width || 0) * (metadata.height || 0) * 4; // Assume RGBA
        this.currentSize += size;
        this.metadata.set(`texture:${id}`, { size, type: 'texture' });
        
        this.evictIfNeeded();
    }
    
    /**
     * Get a cached texture
     */
    getTexture(id) {
        const tex = this.textures.get(id);
        if (tex) {
            tex.lastUsed = Date.now();
            tex.useCount++;
            return tex.texture;
        }
        return null;
    }
    
    /**
     * Remove a shader from cache
     */
    removeShader(id) {
        const shader = this.shaders.get(id);
        if (shader) {
            const meta = this.metadata.get(`shader:${id}`);
            if (meta) {
                this.currentSize -= meta.size;
                this.metadata.delete(`shader:${id}`);
            }
            this.shaders.delete(id);
        }
    }
    
    /**
     * Remove a texture from cache
     */
    removeTexture(id) {
        const tex = this.textures.get(id);
        if (tex) {
            const meta = this.metadata.get(`texture:${id}`);
            if (meta) {
                this.currentSize -= meta.size;
                this.metadata.delete(`texture:${id}`);
            }
            this.textures.delete(id);
        }
    }
    
    /**
     * Clear all cached resources
     */
    clear() {
        this.shaders.clear();
        this.textures.clear();
        this.metadata.clear();
        this.currentSize = 0;
    }
    
    /**
     * Evict least recently used items if cache is too large
     */
    evictIfNeeded() {
        if (this.currentSize <= this.maxCacheSize) {
            return;
        }
        
        // Collect all items with their metadata
        const items = [];
        
        for (const [id, shader] of this.shaders) {
            items.push({
                key: `shader:${id}`,
                lastUsed: shader.lastUsed,
                useCount: shader.useCount,
                type: 'shader'
            });
        }
        
        for (const [id, tex] of this.textures) {
            items.push({
                key: `texture:${id}`,
                lastUsed: tex.lastUsed,
                useCount: tex.useCount,
                type: 'texture'
            });
        }
        
        // Sort by LRU (least recently used first)
        items.sort((a, b) => {
            // Prioritize by use count first, then by last used time
            if (a.useCount !== b.useCount) {
                return a.useCount - b.useCount;
            }
            return a.lastUsed - b.lastUsed;
        });
        
        // Evict until we're under the limit
        for (const item of items) {
            if (this.currentSize <= this.maxCacheSize * 0.8) { // Keep 20% buffer
                break;
            }
            
            const id = item.key.split(':')[1];
            if (item.type === 'shader') {
                this.removeShader(id);
            } else {
                this.removeTexture(id);
            }
            
            console.log(`Evicted ${item.type} ${id} from cache`);
        }
    }
    
    /**
     * Get cache statistics
     */
    getStats() {
        return {
            shaderCount: this.shaders.size,
            textureCount: this.textures.size,
            totalSize: this.currentSize,
            maxSize: this.maxCacheSize,
            utilizationPercent: Math.round((this.currentSize / this.maxCacheSize) * 100)
        };
    }
}