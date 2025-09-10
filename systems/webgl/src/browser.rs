use playground_core_types::Handle;
use playground_core_server::dashboard::{Dashboard, LogLevel};

/// Browser page builder for WebGL renderer
#[derive(Clone)]
pub struct BrowserBuilder {
    dashboard: Option<Handle<Dashboard>>,
}

impl BrowserBuilder {
    pub fn new() -> Self {
        Self {
            dashboard: None,
        }
    }
    
    /// Set the Dashboard for logging
    pub fn set_dashboard(&mut self, dashboard: Handle<Dashboard>) {
        self.dashboard = Some(dashboard);
    }
    
    /// Log a message with component-specific logging
    async fn log(&self, level: LogLevel, message: String) {
        if let Some(ref dashboard) = self.dashboard {
            dashboard.log_component("systems/webgl/browser", level, message, None).await;
        }
    }
    
    /// Generate the main index.html for the WebGL application
    pub async fn generate_index_html(&self) -> String {
        self.log(LogLevel::Info, "Generating WebGL index.html".to_string()).await;
        
        format!(r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0, maximum-scale=1.0, user-scalable=no">
    <meta name="apple-mobile-web-app-capable" content="yes">
    <meta name="mobile-web-app-capable" content="yes">
    <title>Playground WebGL</title>
    <style>
        * {{
            margin: 0;
            padding: 0;
            box-sizing: border-box;
        }}
        body {{
            font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
            background: #000;
            color: #fff;
            overflow: hidden;
            touch-action: none;
        }}
        #canvas {{
            width: 100vw;
            height: 100vh;
            display: block;
        }}
        #status {{
            position: absolute;
            top: 10px;
            left: 10px;
            background: rgba(0, 0, 0, 0.7);
            padding: 10px;
            border-radius: 5px;
            font-size: 12px;
            font-family: monospace;
            z-index: 1000;
        }}
        .connected {{ color: #0f0; }}
        .disconnected {{ color: #f00; }}
        .connecting {{ color: #ff0; }}
    </style>
</head>
<body>
    <div id="status" class="connecting">Initializing WebGL...</div>
    <canvas id="canvas"></canvas>
    <script src="/webgl/renderer.js"></script>
</body>
</html>"#)
    }
    
    /// Generate the WebGL renderer JavaScript
    pub async fn generate_renderer_js(&self) -> String {
        self.log(LogLevel::Info, "Generating WebGL renderer.js".to_string()).await;
        
        format!(r#"
// WebGL Renderer for Playground
class PlaygroundRenderer {{
    constructor() {{
        this.canvas = document.getElementById('canvas');
        this.gl = null;
        this.ws = null;
        this.channelId = null;
        this.channels = {{}};
        this.status = document.getElementById('status');
        
        this.initWebGL();
        this.connectWebSocket();
    }}
    
    initWebGL() {{
        try {{
            this.gl = this.canvas.getContext('webgl2', {{
                alpha: false,
                antialias: false,
                depth: true,
                stencil: false,
                premultipliedAlpha: false,
                preserveDrawingBuffer: false,
                powerPreference: 'high-performance'
            }});
            
            if (!this.gl) {{
                throw new Error('WebGL2 not supported');
            }}
            
            this.resize();
            window.addEventListener('resize', () => this.resize());
            
            this.updateStatus('WebGL2 initialized', 'connecting');
            console.log('[WebGL] Renderer initialized');
        }} catch (e) {{
            console.error('[WebGL] Failed to initialize:', e);
            this.updateStatus('WebGL initialization failed: ' + e.message, 'disconnected');
        }}
    }}
    
    connectWebSocket() {{
        const protocol = window.location.protocol === 'https:' ? 'wss:' : 'ws:';
        const wsUrl = `${{protocol}}//${{window.location.host}}/ws`;
        
        console.log('[WebGL] Connecting to WebSocket:', wsUrl);
        this.updateStatus('Connecting to ' + wsUrl, 'connecting');
        
        try {{
            this.ws = new WebSocket(wsUrl);
            this.ws.binaryType = 'arraybuffer';
        }} catch (e) {{
            console.error('[WebGL] Failed to create WebSocket:', e);
            this.updateStatus('Failed to create WebSocket: ' + e.message, 'disconnected');
            return;
        }}
        
        this.ws.onopen = () => {{
            console.log('[WebGL] WebSocket connected');
            this.updateStatus('Connected', 'connected');
            this.requestChannelManifest();
        }};
        
        this.ws.onmessage = (event) => {{
            this.handleMessage(event.data);
        }};
        
        this.ws.onerror = (error) => {{
            console.error('[WebGL] WebSocket error:', error);
            this.updateStatus('Connection error', 'disconnected');
        }};
        
        this.ws.onclose = () => {{
            console.log('[WebGL] WebSocket disconnected');
            this.updateStatus('Disconnected', 'disconnected');
            setTimeout(() => this.connectWebSocket(), 3000);
        }};
    }}
    
    requestChannelManifest() {{
        // Request channel manifest on control channel (0)
        const message = new ArrayBuffer(11);
        const view = new DataView(message);
        view.setUint16(0, 0, true); // channel_id = 0
        view.setUint16(2, 8, true); // packet_type = 8 (request manifest)
        view.setUint8(4, 0); // priority = 0
        view.setUint32(5, 0, true); // payload_size = 0
        this.ws.send(message);
        console.log('[WebGL] Requested channel manifest');
    }}
    
    handleMessage(data) {{
        const view = new DataView(data);
        const channelId = view.getUint16(0, true);
        const packetType = view.getUint16(2, true);
        const priority = view.getUint8(4);
        const payloadSize = view.getUint32(5, true);
        
        if (channelId === 0 && packetType === 9) {{
            // Channel manifest response
            const payload = new Uint8Array(data, 9, payloadSize);
            const text = new TextDecoder().decode(payload);
            const manifest = JSON.parse(text);
            this.handleChannelManifest(manifest);
        }} else if (this.channelId && channelId === this.channelId) {{
            // Render commands for our channel
            this.handleRenderCommands(data);
        }}
    }}
    
    handleChannelManifest(manifest) {{
        console.log('[WebGL] Received channel manifest:', manifest);
        this.channels = manifest.channels || {{}};
        
        // Find our WebGL renderer channel
        for (const [name, id] of Object.entries(this.channels)) {{
            if (name === 'webgl' || name === 'renderer') {{
                this.channelId = id;
                console.log(`[WebGL] Using channel ${{id}} for rendering`);
                this.subscribeToChannel(id);
                break;
            }}
        }}
    }}
    
    subscribeToChannel(channelId) {{
        // Subscribe to render channel
        const message = new ArrayBuffer(11);
        const view = new DataView(message);
        view.setUint16(0, 0, true); // channel_id = 0
        view.setUint16(2, 1, true); // packet_type = 1 (subscribe)
        view.setUint8(4, 0); // priority
        view.setUint32(5, 2, true); // payload_size
        
        const payload = new ArrayBuffer(2);
        const payloadView = new DataView(payload);
        payloadView.setUint16(0, channelId, true);
        
        const combined = new Uint8Array(11 + 2);
        combined.set(new Uint8Array(message), 0);
        combined.set(new Uint8Array(payload), 11);
        
        this.ws.send(combined.buffer);
        console.log(`[WebGL] Subscribed to channel ${{channelId}}`);
    }}
    
    handleRenderCommands(data) {{
        // Parse and execute render commands
        // This would deserialize the RenderCommandBatch and execute each command
        console.log('[WebGL] Received render commands');
    }}
    
    resize() {{
        const dpr = window.devicePixelRatio || 1;
        const width = window.innerWidth;
        const height = window.innerHeight;
        
        this.canvas.width = width * dpr;
        this.canvas.height = height * dpr;
        this.canvas.style.width = width + 'px';
        this.canvas.style.height = height + 'px';
        
        if (this.gl) {{
            this.gl.viewport(0, 0, this.canvas.width, this.canvas.height);
        }}
        
        console.log(`[WebGL] Resized to ${{width}}x${{height}} (DPR: ${{dpr}})`);
    }}
    
    updateStatus(message, className) {{
        this.status.textContent = message;
        this.status.className = className;
    }}
}}

// Initialize renderer when page loads
window.addEventListener('DOMContentLoaded', () => {{
    window.renderer = new PlaygroundRenderer();
}});
"#)
    }
}