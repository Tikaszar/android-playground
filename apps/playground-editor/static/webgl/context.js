/**
 * WebGL2 Context Management
 * Handles WebGL2 context creation, loss/restore, and viewport management
 */

export class WebGLContextManager {
    constructor(canvas) {
        this.canvas = canvas;
        this.gl = null;
        this.isContextLost = false;
        this.devicePixelRatio = window.devicePixelRatio || 1;
        this.viewport = { x: 0, y: 0, width: 0, height: 0 };
        this.clearColor = [0.133, 0.137, 0.153, 1.0]; // Discord dark background
        
        this.contextLostHandler = null;
        this.contextRestoredHandler = null;
        
        this.initialize();
    }
    
    initialize() {
        // Get WebGL2 context with optimal settings
        const contextAttributes = {
            alpha: false,
            depth: true,
            stencil: false,
            antialias: false,
            premultipliedAlpha: false,
            preserveDrawingBuffer: false,
            powerPreference: 'high-performance',
            failIfMajorPerformanceCaveat: false
        };
        
        this.gl = this.canvas.getContext('webgl2', contextAttributes);
        
        if (!this.gl) {
            throw new Error('WebGL2 is not supported on this device');
        }
        
        // Set up context loss/restore handlers
        this.setupContextHandlers();
        
        // Configure initial GL state
        this.configureGLState();
        
        // Update viewport to match canvas
        this.updateViewport();
    }
    
    setupContextHandlers() {
        this.contextLostHandler = (event) => {
            event.preventDefault();
            this.isContextLost = true;
            console.error('WebGL context lost');
        };
        
        this.contextRestoredHandler = () => {
            this.isContextLost = false;
            console.log('WebGL context restored');
            this.initialize();
        };
        
        this.canvas.addEventListener('webglcontextlost', this.contextLostHandler);
        this.canvas.addEventListener('webglcontextrestored', this.contextRestoredHandler);
    }
    
    configureGLState() {
        const gl = this.gl;
        
        // Enable blending for transparency
        gl.enable(gl.BLEND);
        gl.blendFunc(gl.SRC_ALPHA, gl.ONE_MINUS_SRC_ALPHA);
        
        // Disable depth testing for 2D rendering
        gl.disable(gl.DEPTH_TEST);
        
        // Enable scissor test for clip rects
        gl.enable(gl.SCISSOR_TEST);
        
        // Set clear color
        gl.clearColor(
            this.clearColor[0],
            this.clearColor[1],
            this.clearColor[2],
            this.clearColor[3]
        );
        
        // Enable vertex array objects
        if (gl.getExtension('OES_vertex_array_object') === null) {
            console.warn('VAO extension not available');
        }
    }
    
    updateViewport() {
        // Get canvas display size
        const displayWidth = this.canvas.clientWidth;
        const displayHeight = this.canvas.clientHeight;
        
        // Calculate actual pixel size with device pixel ratio
        const pixelWidth = Math.floor(displayWidth * this.devicePixelRatio);
        const pixelHeight = Math.floor(displayHeight * this.devicePixelRatio);
        
        // Resize canvas backing store if needed
        if (this.canvas.width !== pixelWidth || this.canvas.height !== pixelHeight) {
            this.canvas.width = pixelWidth;
            this.canvas.height = pixelHeight;
        }
        
        // Update viewport
        this.viewport = {
            x: 0,
            y: 0,
            width: pixelWidth,
            height: pixelHeight
        };
        
        // Set GL viewport and scissor
        this.gl.viewport(0, 0, pixelWidth, pixelHeight);
        this.gl.scissor(0, 0, pixelWidth, pixelHeight);
    }
    
    clear() {
        if (this.isContextLost) return;
        
        this.gl.clear(this.gl.COLOR_BUFFER_BIT);
    }
    
    setClearColor(r, g, b, a) {
        this.clearColor = [r, g, b, a];
        this.gl.clearColor(r, g, b, a);
    }
    
    setScissor(x, y, width, height) {
        if (this.isContextLost) return;
        
        // Convert from top-left origin to bottom-left origin
        const flippedY = this.viewport.height - (y + height);
        
        // Apply device pixel ratio
        const scaledX = Math.floor(x * this.devicePixelRatio);
        const scaledY = Math.floor(flippedY * this.devicePixelRatio);
        const scaledWidth = Math.floor(width * this.devicePixelRatio);
        const scaledHeight = Math.floor(height * this.devicePixelRatio);
        
        this.gl.scissor(scaledX, scaledY, scaledWidth, scaledHeight);
    }
    
    resetScissor() {
        if (this.isContextLost) return;
        
        this.gl.scissor(0, 0, this.viewport.width, this.viewport.height);
    }
    
    getProjectionMatrix() {
        // Create orthographic projection matrix for 2D rendering
        // Maps screen coordinates to clip space
        const width = this.viewport.width / this.devicePixelRatio;
        const height = this.viewport.height / this.devicePixelRatio;
        
        return new Float32Array([
            2.0 / width, 0, 0,
            0, -2.0 / height, 0,
            -1.0, 1.0, 1.0
        ]);
    }
    
    dispose() {
        if (this.contextLostHandler) {
            this.canvas.removeEventListener('webglcontextlost', this.contextLostHandler);
        }
        if (this.contextRestoredHandler) {
            this.canvas.removeEventListener('webglcontextrestored', this.contextRestoredHandler);
        }
        
        // Lose context to free GPU resources
        const loseContext = this.gl.getExtension('WEBGL_lose_context');
        if (loseContext) {
            loseContext.loseContext();
        }
        
        this.gl = null;
    }
    
    isReady() {
        return this.gl !== null && !this.isContextLost;
    }
}