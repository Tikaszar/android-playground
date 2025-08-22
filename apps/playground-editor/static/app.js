/**
 * Playground Editor - UI Framework Client
 * 
 * This is a thin client that connects to the UI Framework Plugin
 * which handles all UI rendering and mobile interactions.
 */

import { WebGLRenderer } from './webgl/renderer.js';

class UIFrameworkClient {
    constructor() {
        this.ws = null;
        this.canvas = null;
        this.renderer = null; // WebGL renderer instead of 2D context
        this.channels = new Set();
        this.reconnectAttempts = 0;
        this.maxReconnectAttempts = 10;
        this.reconnectDelay = 1000;
        
        // UI Framework Plugin channels (1200-1209)
        this.UI_FRAMEWORK_BASE = 1200;
        this.UI_FRAMEWORK_CHANNELS = 10;
        
        // Message types for UI Framework
        this.MSG_TYPES = {
            // Incoming from server
            RENDER_FRAME: 1,
            UPDATE_UI: 2,
            SHOW_COMPONENT: 3,
            HIDE_COMPONENT: 4,
            UPDATE_CHAT: 5,
            
            // Outgoing to server
            TOUCH_START: 100,
            TOUCH_MOVE: 101,
            TOUCH_END: 102,
            KEY_DOWN: 103,
            KEY_UP: 104,
            TEXT_INPUT: 105,
            GESTURE: 106,
            RESIZE: 107,
        };
        
        this.init();
    }
    
    init() {
        // Get canvas element
        this.canvas = document.getElementById('canvas');
        if (!this.canvas) {
            console.error('Canvas element not found');
            return;
        }
        
        // Create WebGL renderer
        try {
            this.renderer = new WebGLRenderer(this.canvas);
            console.log('WebGL renderer initialized');
        } catch (error) {
            console.error('Failed to initialize WebGL renderer:', error);
            this.showError();
            return;
        }
        
        // Set up event listeners
        this.setupEventListeners();
        
        // Connect to WebSocket
        this.connect();
        
        // Handle resize
        this.handleResize();
        window.addEventListener('resize', () => this.handleResize());
        window.addEventListener('orientationchange', () => this.handleResize());
    }
    
    setupEventListeners() {
        // Touch events for mobile
        this.canvas.addEventListener('touchstart', (e) => this.handleTouch(e, 'start'), { passive: false });
        this.canvas.addEventListener('touchmove', (e) => this.handleTouch(e, 'move'), { passive: false });
        this.canvas.addEventListener('touchend', (e) => this.handleTouch(e, 'end'), { passive: false });
        this.canvas.addEventListener('touchcancel', (e) => this.handleTouch(e, 'end'), { passive: false });
        
        // Mouse events for desktop
        this.canvas.addEventListener('mousedown', (e) => this.handleMouse(e, 'start'));
        this.canvas.addEventListener('mousemove', (e) => this.handleMouse(e, 'move'));
        this.canvas.addEventListener('mouseup', (e) => this.handleMouse(e, 'end'));
        
        // Keyboard events
        window.addEventListener('keydown', (e) => this.handleKeyboard(e, 'down'));
        window.addEventListener('keyup', (e) => this.handleKeyboard(e, 'up'));
        
        // Prevent default gestures
        document.addEventListener('gesturestart', (e) => e.preventDefault());
        document.addEventListener('gesturechange', (e) => e.preventDefault());
        document.addEventListener('gestureend', (e) => e.preventDefault());
    }
    
    connect() {
        const protocol = window.location.protocol === 'https:' ? 'wss:' : 'ws:';
        const wsUrl = `${protocol}//${window.location.host}/ws`;
        
        console.log(`=== WebSocket Connection Attempt ===`);
        console.log(`URL: ${wsUrl}`);
        console.log(`Location host: ${window.location.host}`);
        console.log(`Location protocol: ${window.location.protocol}`);
        console.log(`Browser URL: ${window.location.href}`);
        
        try {
            this.ws = new WebSocket(wsUrl);
            this.ws.binaryType = 'arraybuffer';
            
            console.log('WebSocket object created successfully');
            console.log('Initial readyState:', this.ws.readyState);
            
            this.ws.onopen = () => {
                console.log('WebSocket onopen event fired');
                this.onConnect();
            };
            this.ws.onmessage = (e) => this.onMessage(e);
            this.ws.onerror = (e) => {
                console.error('WebSocket onerror event fired');
                this.onError(e);
            };
            this.ws.onclose = (e) => {
                console.log('WebSocket onclose event fired', e.code, e.reason);
                this.onDisconnect();
            };
        } catch (error) {
            console.error('Failed to create WebSocket:', error);
            console.error('Error stack:', error.stack);
            this.showError();
        }
    }
    
    onConnect() {
        console.log('Connected to UI Framework');
        this.reconnectAttempts = 0;
        
        // Browser is a client - no need to register channels
        // The UI Framework Plugin on the server handles our messages
        
        // Hide loading, show canvas
        document.body.classList.add('ui-ready');
        document.getElementById('loading').style.display = 'none';
        document.getElementById('error').style.display = 'none';
        this.canvas.style.display = 'block';
        
        // Wait a bit for WebSocket to fully stabilize before sending first message
        // This avoids race condition with async connection establishment
        
        // Send connection log to server
        this.sendLog('info', 'Browser connected and ready');
        setTimeout(() => {
            // Send initial resize event to UI Framework Plugin
            this.handleResize();
        }, 100);
    }
    
    onMessage(event) {
        if (event.data instanceof ArrayBuffer) {
            this.handleBinaryMessage(event.data);
        } else {
            try {
                const msg = JSON.parse(event.data);
                this.handleJsonMessage(msg);
            } catch (e) {
                console.error('Failed to parse message:', e);
            }
        }
    }
    
    handleBinaryMessage(data) {
        const view = new DataView(data);
        
        // Parse packet header (big-endian to match server)
        const channelId = view.getUint16(0, false);
        const packetType = view.getUint16(2, false);
        const priority = view.getUint8(4);
        const payloadSize = view.getUint32(5, false);  // Note: offset 5 is correct per Rust code
        
        // Check if this is the UI channel (channel 10)
        if (channelId === 10) {
            // Extract payload
            const payload = new Uint8Array(data, 9, payloadSize);
            
            this.sendLog('debug', `Received packet on channel 10: type ${packetType}, size ${payloadSize}`);
            
            // Handle UI system packets (RenderBatch is type 104)
            if (packetType === 104) {
                this.renderFrame(payload);
            } else {
                console.log(`UI packet type ${packetType} on channel ${channelId}`);
                this.sendLog('warning', `Unknown UI packet type ${packetType} on channel ${channelId}`);
            }
        }
        // Check if this is a UI Framework channel
        else if (channelId >= this.UI_FRAMEWORK_BASE && 
                 channelId < this.UI_FRAMEWORK_BASE + this.UI_FRAMEWORK_CHANNELS) {
            
            // Extract payload
            const payload = new Uint8Array(data, 9, payloadSize);
            
            // Handle based on packet type
            switch (packetType) {
                case this.MSG_TYPES.RENDER_FRAME:
                    this.renderFrame(payload);
                    break;
                case this.MSG_TYPES.UPDATE_UI:
                    this.updateUI(payload);
                    break;
                case this.MSG_TYPES.SHOW_COMPONENT:
                    this.showComponent(payload);
                    break;
                case this.MSG_TYPES.UPDATE_CHAT:
                    this.updateChat(payload);
                    break;
                default:
                    console.log(`Unknown packet type ${packetType} on channel ${channelId}`);
            }
        }
    }
    
    handleJsonMessage(msg) {
        // Handle control messages
        if (msg.type === 'channel_registered') {
            console.log(`Registered for channel ${msg.channel_id}`);
            this.channels.add(msg.channel_id);
        }
    }
    
    renderFrame(payload) {
        // Parse the bincode-serialized render batch message
        try {
            const batch = this.deserializeBincode(payload);
            console.log('Received render batch:', batch);
            this.sendLog('debug', `Received render batch: frame ${batch.frame_id}, ${batch.commands.length} commands`);
            
            if (batch && batch.commands && this.renderer) {
                console.log(`Executing ${batch.commands.length} render commands`);
                this.sendLog('debug', `Commands: ${JSON.stringify(batch.commands)}`);
                // Use WebGL renderer to execute commands
                this.renderer.executeCommandBatch(batch);
            } else if (!this.renderer) {
                console.error('WebGL renderer not initialized!');
                this.sendLog('error', 'WebGL renderer not initialized!');
            }
        } catch (e) {
            console.error('Failed to parse render batch:', e);
            this.sendLog('error', `Failed to parse render batch: ${e.message}`);
        }
    }
    
    // Send log message to server on control channel
    sendLog(level, message) {
        if (!this.ws || this.ws.readyState !== WebSocket.OPEN) return;
        
        // Send log message on control channel (0) with a special packet type
        const logData = JSON.stringify({
            type: 'browser_log',
            level: level,
            message: message,
            timestamp: Date.now()
        });
        
        const encoder = new TextEncoder();
        const payload = encoder.encode(logData);
        
        // Create packet (channel 0, type 200 for browser logs)
        const packet = new ArrayBuffer(9 + payload.length);
        const view = new DataView(packet);
        
        // Header (big-endian to match server)
        view.setUint16(0, 0, false);        // Channel 0 (control)
        view.setUint16(2, 200, false);      // Type 200 (browser log)
        view.setUint8(4, 1);                // Priority medium
        view.setUint32(5, payload.length, false); // Payload size
        
        // Payload
        const uint8View = new Uint8Array(packet);
        uint8View.set(payload, 9);
        
        try {
            this.ws.send(packet);
        } catch (e) {
            console.error('Failed to send log to server:', e);
        }
    }
    
    // Deserialize bincode format (simplified for RenderCommandBatch)
    deserializeBincode(payload) {
        this.sendLog('debug', `Deserializing bincode payload: ${payload.length} bytes`);
        
        // Log first few bytes for debugging
        const firstBytes = Array.from(payload.slice(0, Math.min(32, payload.length)))
            .map(b => b.toString(16).padStart(2, '0'))
            .join(' ');
        this.sendLog('debug', `First bytes: ${firstBytes}`);
        
        const view = new DataView(payload.buffer, payload.byteOffset, payload.byteLength);
        let offset = 0;
        
        // RenderCommandBatch fields are serialized in declaration order:
        // 1. commands: Vec<RenderCommand>
        // 2. viewport: Option<Viewport>  
        // 3. frame_id: u64
        
        // Read commands vector length (u64 for bincode)
        if (offset + 8 > payload.length) {
            throw new Error(`Not enough bytes for commands length at offset ${offset}`);
        }
        const commandsLen = Number(view.getBigUint64(offset, true)); // little-endian
        offset += 8;
        this.sendLog('debug', `Commands count: ${commandsLen}`);
        
        // Sanity check
        if (commandsLen > 1000) {
            throw new Error(`Unreasonable commands count: ${commandsLen}`);
        }
        
        const commands = [];
        for (let i = 0; i < commandsLen; i++) {
            // Read command variant index (bincode uses u32 for enum discriminants)
            if (offset + 4 > payload.length) {
                throw new Error(`Not enough bytes for variant at offset ${offset}, need 4, have ${payload.length - offset}`);
            }
            const variantIndex = view.getUint32(offset, true);
            offset += 4;
            this.sendLog('debug', `Command ${i}: variant ${variantIndex} at offset ${offset-4}`);
            
            // Parse command based on variant
            let command;
            switch (variantIndex) {
                case 0: // Clear
                    command = {
                        Clear: {
                            color: [
                                view.getFloat32(offset, true),
                                view.getFloat32(offset + 4, true),
                                view.getFloat32(offset + 8, true),
                                view.getFloat32(offset + 12, true)
                            ]
                        }
                    };
                    offset += 16;
                    break;
                    
                case 1: // DrawQuad
                    command = {
                        DrawQuad: {
                            position: [
                                view.getFloat32(offset, true),
                                view.getFloat32(offset + 4, true)
                            ],
                            size: [
                                view.getFloat32(offset + 8, true),
                                view.getFloat32(offset + 12, true)
                            ],
                            color: [
                                view.getFloat32(offset + 16, true),
                                view.getFloat32(offset + 20, true),
                                view.getFloat32(offset + 24, true),
                                view.getFloat32(offset + 28, true)
                            ]
                        }
                    };
                    offset += 32;
                    break;
                    
                case 6: // SetClipRect
                    command = {
                        SetClipRect: {
                            position: [
                                view.getFloat32(offset, true),
                                view.getFloat32(offset + 4, true)
                            ],
                            size: [
                                view.getFloat32(offset + 8, true),
                                view.getFloat32(offset + 12, true)
                            ]
                        }
                    };
                    offset += 16;
                    break;
                    
                case 7: // ClearClipRect
                    command = { ClearClipRect: {} };
                    // No data to read
                    break;
                    
                case 10: // PushState
                    command = { PushState: {} };
                    // No data to read
                    break;
                    
                case 11: // PopState
                    command = { PopState: {} };
                    // No data to read
                    break;
                    
                // Add more command types as needed
                default:
                    throw new Error(`Unknown command variant: ${variantIndex} at offset ${offset-4}`);
            }
            
            if (command) {
                commands.push(command);
            }
        }
        
        // Read viewport: Option<Viewport>
        const hasViewport = view.getUint8(offset);
        offset += 1;
        let viewport = null;
        if (hasViewport === 1) {
            viewport = {
                x: view.getUint32(offset, true),
                y: view.getUint32(offset + 4, true),
                width: view.getUint32(offset + 8, true),
                height: view.getUint32(offset + 12, true)
            };
            offset += 16;
        }
        
        // Read frame_id (u64) - comes last in the struct
        const frameId = Number(view.getBigUint64(offset, true));
        offset += 8;
        
        this.sendLog('debug', `Parsed batch: frameId=${frameId}, commands=${commandsLen}, viewport=${viewport ? 'yes' : 'no'}`);
        
        return {
            frame_id: frameId,
            commands: commands,
            viewport: viewport
        };
    }
    
    // executeRenderCommands removed - now handled by WebGL renderer
    
    updateUI(payload) {
        // UI Framework is updating UI elements
        const decoder = new TextDecoder();
        const data = JSON.parse(decoder.decode(payload));
        console.log('UI Update:', data);
    }
    
    showComponent(payload) {
        // Show an inline component
        const decoder = new TextDecoder();
        const data = JSON.parse(decoder.decode(payload));
        console.log('Show Component:', data);
    }
    
    updateChat(payload) {
        // Update chat messages
        const decoder = new TextDecoder();
        const data = JSON.parse(decoder.decode(payload));
        console.log('Chat Update:', data);
    }
    
    handleTouch(event, type) {
        event.preventDefault();
        
        const rect = this.canvas.getBoundingClientRect();
        const touches = [];
        
        for (let i = 0; i < event.touches.length; i++) {
            const touch = event.touches[i];
            touches.push({
                id: touch.identifier,
                x: (touch.clientX - rect.left) * (this.canvas.width / rect.width),
                y: (touch.clientY - rect.top) * (this.canvas.height / rect.height),
                force: touch.force || 1.0,
                radiusX: touch.radiusX || 1,
                radiusY: touch.radiusY || 1,
            });
        }
        
        let msgType;
        switch (type) {
            case 'start': msgType = this.MSG_TYPES.TOUCH_START; break;
            case 'move': msgType = this.MSG_TYPES.TOUCH_MOVE; break;
            case 'end': msgType = this.MSG_TYPES.TOUCH_END; break;
        }
        
        this.sendToUIFramework(msgType, {
            touches,
            timestamp: Date.now(),
        });
    }
    
    handleMouse(event, type) {
        const rect = this.canvas.getBoundingClientRect();
        const x = (event.clientX - rect.left) * (this.canvas.width / rect.width);
        const y = (event.clientY - rect.top) * (this.canvas.height / rect.height);
        
        // Convert mouse to touch for UI Framework
        let msgType;
        switch (type) {
            case 'start': msgType = this.MSG_TYPES.TOUCH_START; break;
            case 'move': msgType = this.MSG_TYPES.TOUCH_MOVE; break;
            case 'end': msgType = this.MSG_TYPES.TOUCH_END; break;
        }
        
        this.sendToUIFramework(msgType, {
            touches: [{
                id: 0,
                x,
                y,
                force: 1.0,
                radiusX: 1,
                radiusY: 1,
            }],
            timestamp: Date.now(),
            button: event.button,
        });
    }
    
    handleKeyboard(event, type) {
        const msgType = type === 'down' ? this.MSG_TYPES.KEY_DOWN : this.MSG_TYPES.KEY_UP;
        
        this.sendToUIFramework(msgType, {
            key: event.key,
            code: event.code,
            ctrlKey: event.ctrlKey,
            shiftKey: event.shiftKey,
            altKey: event.altKey,
            metaKey: event.metaKey,
            repeat: event.repeat,
            timestamp: Date.now(),
        });
    }
    
    handleResize() {
        // Get device pixel ratio for high DPI displays
        const dpr = window.devicePixelRatio || 1;
        
        // Get display size
        const rect = this.canvas.getBoundingClientRect();
        const width = rect.width;
        const height = rect.height;
        
        // Update WebGL renderer viewport
        if (this.renderer) {
            this.renderer.resize();
        }
        
        // Notify UI Framework of resize
        this.sendToUIFramework(this.MSG_TYPES.RESIZE, {
            width: rect.width,
            height: rect.height,
            dpr,
            orientation: window.orientation || 0,
        });
    }
    
    sendToUIFramework(msgType, data) {
        if (!this.ws || this.ws.readyState !== WebSocket.OPEN) {
            console.warn('WebSocket not ready, skipping message type:', msgType);
            return;
        }
        
        // Send to UI Framework main channel
        const channel = this.UI_FRAMEWORK_BASE;
        const payload = JSON.stringify(data);
        const encoder = new TextEncoder();
        const payloadBytes = encoder.encode(payload);
        
        // Create binary packet
        const packet = new ArrayBuffer(9 + payloadBytes.length);
        const view = new DataView(packet);
        
        // Write header (big-endian to match server)
        view.setUint16(0, channel, false);
        view.setUint16(2, msgType, false);
        view.setUint8(4, 2); // Priority: High
        view.setUint32(5, payloadBytes.length, false);
        
        // Write payload
        const bytes = new Uint8Array(packet, 9);
        bytes.set(payloadBytes);
        
        this.ws.send(packet);
    }
    
    // Browser doesn't need to register channels - it's a client
    // The UI Framework Plugin (server-side) handles messages on channels 1200-1209
    // We just send messages to those channels
    
    onError(error) {
        console.error('WebSocket error:', error);
        console.error('WebSocket readyState:', this.ws?.readyState);
        console.error('WebSocket URL:', this.ws?.url);
        console.error('Error type:', error.type);
        console.error('Error target:', error.target);
        
        // Also log to server if possible
        if (this.ws && this.ws.readyState === WebSocket.OPEN) {
            this.sendLog('error', `WebSocket error: ${error.type || 'unknown'}`);
        }
    }
    
    onDisconnect() {
        console.log('Disconnected from UI Framework');
        document.body.classList.remove('ui-ready');
        
        // Try to reconnect
        if (this.reconnectAttempts < this.maxReconnectAttempts) {
            this.reconnectAttempts++;
            const delay = Math.min(this.reconnectDelay * Math.pow(1.5, this.reconnectAttempts - 1), 30000);
            
            console.log(`Reconnecting in ${delay}ms (attempt ${this.reconnectAttempts})`);
            setTimeout(() => this.connect(), delay);
        } else {
            this.showError();
        }
    }
    
    showError() {
        document.getElementById('loading').style.display = 'none';
        document.getElementById('error').style.display = 'block';
        this.canvas.style.display = 'none';
    }
}

// Global reconnect function
function reconnect() {
    window.uiClient.reconnectAttempts = 0;
    window.uiClient.connect();
    document.getElementById('error').style.display = 'none';
    document.getElementById('loading').style.display = 'block';
}

// Initialize when DOM is ready
if (document.readyState === 'loading') {
    document.addEventListener('DOMContentLoaded', () => {
        window.uiClient = new UIFrameworkClient();
    });
} else {
    window.uiClient = new UIFrameworkClient();
}