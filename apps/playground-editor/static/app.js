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
        
        console.log(`Connecting to UI Framework at ${wsUrl}`);
        
        try {
            this.ws = new WebSocket(wsUrl);
            this.ws.binaryType = 'arraybuffer';
            
            this.ws.onopen = () => this.onConnect();
            this.ws.onmessage = (e) => this.onMessage(e);
            this.ws.onerror = (e) => this.onError(e);
            this.ws.onclose = () => this.onDisconnect();
        } catch (error) {
            console.error('Failed to create WebSocket:', error);
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
            
            // Handle UI system packets (RenderBatch is type 104)
            if (packetType === 104) {
                this.renderFrame(payload);
            } else {
                console.log(`UI packet type ${packetType} on channel ${channelId}`);
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
        // Parse the render batch message
        try {
            const decoder = new TextDecoder();
            const batch = JSON.parse(decoder.decode(payload));
            
            if (batch.commands && this.renderer) {
                // Use WebGL renderer to execute commands
                this.renderer.executeCommandBatch(batch);
            }
        } catch (e) {
            console.error('Failed to parse render batch:', e);
        }
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