// Conversational IDE JavaScript Client
// Connects to UI Framework Plugin via WebSocket channels 1200-1209

class ConversationalIDE {
    constructor() {
        this.ws = null;
        this.currentChannel = 'general';
        this.messages = new Map(); // channelId -> messages[]
        this.bubbleStates = new Map(); // messageId -> state
        this.reconnectAttempts = 0;
        this.maxReconnectAttempts = 10;
        this.reconnectDelay = 1000;
        
        // UI Framework Plugin channels
        this.UI_CHANNEL_BASE = 1200;
        this.UI_CHANNEL_RESULTS = 1201;
        
        this.init();
    }

    init() {
        this.connectWebSocket();
        this.setupEventListeners();
        this.loadInitialState();
    }

    connectWebSocket() {
        const protocol = window.location.protocol === 'https:' ? 'wss:' : 'ws:';
        const host = window.location.hostname || 'localhost';
        const port = window.location.port || '8080';
        const wsUrl = `${protocol}//${host}:${port}/ws`;

        console.log('Connecting to WebSocket:', wsUrl);
        this.updateConnectionStatus('connecting');

        try {
            this.ws = new WebSocket(wsUrl);
            this.setupWebSocketHandlers();
        } catch (error) {
            console.error('WebSocket connection error:', error);
            this.scheduleReconnect();
        }
    }

    setupWebSocketHandlers() {
        this.ws.onopen = () => {
            console.log('WebSocket connected');
            this.updateConnectionStatus('connected');
            this.reconnectAttempts = 0;
            
            // Register for UI Framework channels
            this.registerChannel(this.UI_CHANNEL_BASE, 'ui-framework');
            this.registerChannel(this.UI_CHANNEL_RESULTS, 'ui-framework-results');
            
            // Request initial state
            this.sendPacket(this.UI_CHANNEL_BASE, {
                type: 'get_state',
                timestamp: Date.now()
            });
        };

        this.ws.onmessage = (event) => {
            if (event.data instanceof Blob) {
                // Binary message - parse packet
                this.handleBinaryMessage(event.data);
            } else {
                // Text message - parse JSON
                try {
                    const message = JSON.parse(event.data);
                    this.handleTextMessage(message);
                } catch (error) {
                    console.error('Failed to parse message:', error);
                }
            }
        };

        this.ws.onerror = (error) => {
            console.error('WebSocket error:', error);
            this.updateConnectionStatus('error');
        };

        this.ws.onclose = () => {
            console.log('WebSocket disconnected');
            this.updateConnectionStatus('disconnected');
            this.scheduleReconnect();
        };
    }

    async handleBinaryMessage(blob) {
        const buffer = await blob.arrayBuffer();
        const view = new DataView(buffer);
        
        // Parse packet header
        const channelId = view.getUint16(0, true);
        const packetType = view.getUint16(2, true);
        const priority = view.getUint8(4);
        const payloadSize = view.getUint32(5, true);
        
        // Extract payload
        const payloadBytes = new Uint8Array(buffer, 9, payloadSize);
        const payloadText = new TextDecoder().decode(payloadBytes);
        
        try {
            const payload = JSON.parse(payloadText);
            this.handlePacket(channelId, packetType, payload);
        } catch (error) {
            console.error('Failed to parse packet payload:', error);
        }
    }

    handleTextMessage(message) {
        // Handle text-based control messages
        if (message.type === 'channel_registered') {
            console.log('Channel registered:', message.channel_id, message.name);
        } else if (message.type === 'error') {
            console.error('Server error:', message.message);
        }
    }

    handlePacket(channelId, packetType, payload) {
        // Handle packets from UI Framework Plugin
        if (channelId === this.UI_CHANNEL_BASE || channelId === this.UI_CHANNEL_RESULTS) {
            this.handleUIFrameworkPacket(payload);
        }
    }

    handleUIFrameworkPacket(payload) {
        console.log('UI Framework packet:', payload);
        
        switch (payload.type) {
            case 'state_update':
                this.updateState(payload.state);
                break;
                
            case 'message':
                this.addMessage(payload.message);
                break;
                
            case 'channel_update':
                this.updateChannel(payload.channel);
                break;
                
            case 'inline_component':
                this.handleInlineComponent(payload.component);
                break;
                
            case 'agent_status':
                this.updateAgentStatus(payload.status);
                break;
                
            case 'error':
                this.showError(payload.error);
                break;
                
            default:
                console.warn('Unknown UI Framework packet type:', payload.type);
        }
    }

    // Message rendering functions
    addMessage(messageData) {
        const { channel_id, id, author, content, timestamp, components } = messageData;
        
        // Store message
        if (!this.messages.has(channel_id)) {
            this.messages.set(channel_id, []);
        }
        this.messages.get(channel_id).push(messageData);
        
        // Render if current channel
        if (channel_id === this.currentChannel) {
            this.renderMessage(messageData);
        }
    }

    renderMessage(messageData) {
        const messagesContainer = document.getElementById('messages');
        
        const messageEl = document.createElement('div');
        messageEl.className = 'message';
        messageEl.id = `message-${messageData.id}`;
        
        // Avatar
        const avatarEl = document.createElement('div');
        avatarEl.className = 'message-avatar';
        avatarEl.textContent = messageData.author.charAt(0).toUpperCase();
        if (messageData.author_type === 'assistant') {
            avatarEl.style.backgroundColor = '#5865f2';
        } else if (messageData.author_type === 'system') {
            avatarEl.style.backgroundColor = '#ed4245';
        }
        
        // Content
        const contentEl = document.createElement('div');
        contentEl.className = 'message-content';
        
        // Header
        const headerEl = document.createElement('div');
        headerEl.className = 'message-header';
        
        const authorEl = document.createElement('span');
        authorEl.className = 'message-author';
        authorEl.textContent = messageData.author;
        
        const timestampEl = document.createElement('span');
        timestampEl.className = 'message-timestamp';
        timestampEl.textContent = this.formatTimestamp(messageData.timestamp);
        
        headerEl.appendChild(authorEl);
        headerEl.appendChild(timestampEl);
        
        // Text
        const textEl = document.createElement('div');
        textEl.className = 'message-text';
        textEl.innerHTML = this.formatMessageText(messageData.content);
        
        contentEl.appendChild(headerEl);
        contentEl.appendChild(textEl);
        
        // Inline components
        if (messageData.components && messageData.components.length > 0) {
            messageData.components.forEach(component => {
                const componentEl = this.createInlineComponent(component);
                contentEl.appendChild(componentEl);
            });
        }
        
        messageEl.appendChild(avatarEl);
        messageEl.appendChild(contentEl);
        messagesContainer.appendChild(messageEl);
        
        // Scroll to bottom
        messagesContainer.scrollTop = messagesContainer.scrollHeight;
    }

    createInlineComponent(component) {
        const componentEl = document.createElement('div');
        componentEl.className = 'inline-component';
        componentEl.id = `component-${component.id}`;
        
        // Header
        const headerEl = document.createElement('div');
        headerEl.className = 'inline-header';
        headerEl.onclick = () => this.toggleBubbleState(component.id);
        
        const titleEl = document.createElement('div');
        titleEl.className = 'inline-title';
        
        const iconEl = document.createElement('span');
        iconEl.textContent = this.getComponentIcon(component.type);
        
        const nameEl = document.createElement('span');
        nameEl.textContent = component.title || component.type;
        
        titleEl.appendChild(iconEl);
        titleEl.appendChild(nameEl);
        
        // Actions
        const actionsEl = document.createElement('div');
        actionsEl.className = 'inline-actions';
        
        if (component.type === 'editor') {
            const saveBtn = document.createElement('button');
            saveBtn.className = 'inline-action';
            saveBtn.textContent = 'Save';
            saveBtn.onclick = (e) => {
                e.stopPropagation();
                this.saveEditorContent(component.id);
            };
            actionsEl.appendChild(saveBtn);
        }
        
        headerEl.appendChild(titleEl);
        headerEl.appendChild(actionsEl);
        
        // Content
        const contentEl = document.createElement('div');
        contentEl.className = 'inline-content';
        contentEl.id = `content-${component.id}`;
        
        // Set initial bubble state
        const state = this.bubbleStates.get(component.id) || component.initial_state || 'expanded';
        this.bubbleStates.set(component.id, state);
        if (state !== 'expanded') {
            contentEl.classList.add(state);
        }
        
        // Render component content based on type
        switch (component.type) {
            case 'editor':
                contentEl.appendChild(this.createEditor(component));
                break;
            case 'file_browser':
                contentEl.appendChild(this.createFileBrowser(component));
                break;
            case 'terminal':
                contentEl.appendChild(this.createTerminal(component));
                break;
            case 'diff':
                contentEl.appendChild(this.createDiff(component));
                break;
            default:
                contentEl.textContent = JSON.stringify(component.data, null, 2);
        }
        
        componentEl.appendChild(headerEl);
        componentEl.appendChild(contentEl);
        
        return componentEl;
    }

    createEditor(component) {
        const editorEl = document.createElement('div');
        editorEl.className = 'inline-editor';
        editorEl.contentEditable = true;
        editorEl.textContent = component.data.content || '';
        editorEl.dataset.language = component.data.language || 'plaintext';
        editorEl.dataset.filepath = component.data.filepath || '';
        
        // Store editor content for saving
        editorEl.oninput = () => {
            component.data.content = editorEl.textContent;
        };
        
        return editorEl;
    }

    createFileBrowser(component) {
        const browserEl = document.createElement('div');
        browserEl.className = 'inline-file-browser';
        
        const renderTree = (items, level = 0) => {
            items.forEach(item => {
                const itemEl = document.createElement('div');
                itemEl.className = 'file-tree-item';
                itemEl.style.paddingLeft = `${level * 16 + 8}px`;
                
                const iconEl = document.createElement('span');
                iconEl.className = 'file-icon';
                iconEl.textContent = item.type === 'directory' ? '📁' : '📄';
                
                const nameEl = document.createElement('span');
                nameEl.textContent = item.name;
                
                itemEl.appendChild(iconEl);
                itemEl.appendChild(nameEl);
                
                itemEl.onclick = () => {
                    if (item.type === 'file') {
                        this.openFile(item.path);
                    } else {
                        this.toggleDirectory(item.path);
                    }
                };
                
                browserEl.appendChild(itemEl);
                
                if (item.children && item.expanded) {
                    renderTree(item.children, level + 1);
                }
            });
        };
        
        if (component.data.items) {
            renderTree(component.data.items);
        }
        
        return browserEl;
    }

    createTerminal(component) {
        const terminalEl = document.createElement('div');
        terminalEl.className = 'inline-terminal';
        terminalEl.textContent = component.data.output || '$ ';
        
        return terminalEl;
    }

    createDiff(component) {
        const diffEl = document.createElement('div');
        diffEl.className = 'inline-diff';
        
        // Simple diff rendering
        const lines = (component.data.diff || '').split('\n');
        lines.forEach(line => {
            const lineEl = document.createElement('div');
            if (line.startsWith('+')) {
                lineEl.style.color = '#3ba55c';
                lineEl.style.backgroundColor = 'rgba(59, 165, 92, 0.1)';
            } else if (line.startsWith('-')) {
                lineEl.style.color = '#ed4245';
                lineEl.style.backgroundColor = 'rgba(237, 66, 69, 0.1)';
            }
            lineEl.textContent = line;
            diffEl.appendChild(lineEl);
        });
        
        return diffEl;
    }

    getComponentIcon(type) {
        const icons = {
            'editor': '📝',
            'file_browser': '📁',
            'terminal': '💻',
            'diff': '🔄',
            'error': '❌'
        };
        return icons[type] || '📦';
    }

    toggleBubbleState(componentId) {
        const contentEl = document.getElementById(`content-${componentId}`);
        if (!contentEl) return;
        
        const currentState = this.bubbleStates.get(componentId) || 'expanded';
        let newState;
        
        if (currentState === 'expanded') {
            newState = 'compressed';
        } else if (currentState === 'compressed') {
            newState = 'collapsed';
        } else {
            newState = 'expanded';
        }
        
        // Update classes
        contentEl.classList.remove('collapsed', 'compressed');
        if (newState !== 'expanded') {
            contentEl.classList.add(newState);
        }
        
        this.bubbleStates.set(componentId, newState);
        
        // Notify server
        this.sendPacket(this.UI_CHANNEL_BASE, {
            type: 'bubble_state_change',
            component_id: componentId,
            state: newState
        });
    }

    // WebSocket communication
    registerChannel(channelId, name) {
        const packet = {
            type: 0, // CONTROL_REGISTER
            channel_id: channelId,
            name: name
        };
        
        // Send as control message on channel 0
        this.sendPacket(0, packet);
    }

    sendPacket(channelId, payload) {
        if (!this.ws || this.ws.readyState !== WebSocket.OPEN) {
            console.warn('WebSocket not connected');
            return;
        }
        
        const payloadText = JSON.stringify(payload);
        const payloadBytes = new TextEncoder().encode(payloadText);
        
        // Create packet
        const buffer = new ArrayBuffer(9 + payloadBytes.length);
        const view = new DataView(buffer);
        
        // Header
        view.setUint16(0, channelId, true); // channel_id
        view.setUint16(2, 1, true); // packet_type (1 = data)
        view.setUint8(4, 2); // priority (2 = medium)
        view.setUint32(5, payloadBytes.length, true); // payload_size
        
        // Payload
        const uint8View = new Uint8Array(buffer, 9);
        uint8View.set(payloadBytes);
        
        this.ws.send(buffer);
    }

    sendMessage() {
        const input = document.getElementById('message-input');
        const text = input.value.trim();
        
        if (!text) return;
        
        // Send message to server
        this.sendPacket(this.UI_CHANNEL_BASE, {
            type: 'send_message',
            channel: this.currentChannel,
            content: text,
            timestamp: Date.now()
        });
        
        // Clear input
        input.value = '';
        input.style.height = 'auto';
    }

    // UI event handlers
    setupEventListeners() {
        // Channel switching
        document.querySelectorAll('.channel').forEach(channelEl => {
            channelEl.addEventListener('click', () => {
                this.switchChannel(channelEl.dataset.channel);
            });
        });
        
        // Message input
        const input = document.getElementById('message-input');
        input.addEventListener('keydown', (e) => {
            if (e.key === 'Enter' && !e.shiftKey) {
                e.preventDefault();
                this.sendMessage();
            }
        });
        
        // Auto-resize textarea
        input.addEventListener('input', () => {
            input.style.height = 'auto';
            input.style.height = input.scrollHeight + 'px';
        });
    }

    switchChannel(channelId) {
        this.currentChannel = channelId;
        
        // Update UI
        document.querySelectorAll('.channel').forEach(el => {
            el.classList.remove('active');
        });
        document.querySelector(`[data-channel="${channelId}"]`).classList.add('active');
        document.getElementById('current-channel').textContent = `# ${channelId}`;
        document.getElementById('message-input').placeholder = `Message #${channelId}`;
        
        // Clear and reload messages
        const messagesContainer = document.getElementById('messages');
        messagesContainer.innerHTML = '';
        
        const messages = this.messages.get(channelId) || [];
        messages.forEach(msg => this.renderMessage(msg));
    }

    // Utility functions
    formatTimestamp(timestamp) {
        const date = new Date(timestamp);
        const now = new Date();
        const diff = now - date;
        
        if (diff < 60000) {
            return 'just now';
        } else if (diff < 3600000) {
            return `${Math.floor(diff / 60000)}m ago`;
        } else if (diff < 86400000) {
            return `${Math.floor(diff / 3600000)}h ago`;
        } else {
            return date.toLocaleDateString();
        }
    }

    formatMessageText(text) {
        // Convert markdown-like syntax to HTML
        return text
            .replace(/`([^`]+)`/g, '<code>$1</code>')
            .replace(/\*\*([^*]+)\*\*/g, '<strong>$1</strong>')
            .replace(/\*([^*]+)\*/g, '<em>$1</em>')
            .replace(/\n/g, '<br>');
    }

    updateConnectionStatus(status) {
        const statusDot = document.getElementById('connection-status');
        const statusText = document.getElementById('connection-text');
        
        statusDot.className = 'status-dot';
        
        switch (status) {
            case 'connected':
                statusDot.classList.add('connected');
                statusText.textContent = 'Connected';
                break;
            case 'connecting':
                statusDot.classList.add('connecting');
                statusText.textContent = 'Connecting...';
                break;
            case 'disconnected':
                statusDot.classList.add('disconnected');
                statusText.textContent = 'Disconnected';
                break;
            case 'error':
                statusDot.classList.add('disconnected');
                statusText.textContent = 'Connection Error';
                break;
        }
    }

    updateAgentStatus(status) {
        document.getElementById('agent-status').textContent = status;
    }

    showError(error) {
        console.error('UI Framework error:', error);
        // Could show a toast notification here
    }

    scheduleReconnect() {
        if (this.reconnectAttempts >= this.maxReconnectAttempts) {
            console.error('Max reconnection attempts reached');
            return;
        }
        
        this.reconnectAttempts++;
        const delay = Math.min(this.reconnectDelay * Math.pow(1.5, this.reconnectAttempts - 1), 30000);
        
        console.log(`Reconnecting in ${delay}ms (attempt ${this.reconnectAttempts}/${this.maxReconnectAttempts})`);
        setTimeout(() => this.connectWebSocket(), delay);
    }

    loadInitialState() {
        // Load any persisted state from localStorage
        const savedState = localStorage.getItem('conversational-ide-state');
        if (savedState) {
            try {
                const state = JSON.parse(savedState);
                // Restore bubble states, etc.
                if (state.bubbleStates) {
                    this.bubbleStates = new Map(state.bubbleStates);
                }
            } catch (error) {
                console.error('Failed to load saved state:', error);
            }
        }
    }

    saveState() {
        const state = {
            bubbleStates: Array.from(this.bubbleStates.entries()),
            currentChannel: this.currentChannel
        };
        localStorage.setItem('conversational-ide-state', JSON.stringify(state));
    }

    // Additional UI functions
    saveEditorContent(componentId) {
        const editorEl = document.querySelector(`#component-${componentId} .inline-editor`);
        if (!editorEl) return;
        
        const content = editorEl.textContent;
        const filepath = editorEl.dataset.filepath;
        
        this.sendPacket(this.UI_CHANNEL_BASE, {
            type: 'save_file',
            filepath: filepath,
            content: content
        });
    }

    openFile(filepath) {
        this.sendPacket(this.UI_CHANNEL_BASE, {
            type: 'open_file',
            filepath: filepath
        });
    }

    toggleDirectory(path) {
        this.sendPacket(this.UI_CHANNEL_BASE, {
            type: 'toggle_directory',
            path: path
        });
    }
}

// Global functions for HTML onclick handlers
function toggleSidebar() {
    const sidebar = document.getElementById('sidebar');
    sidebar.classList.toggle('open');
}

function toggleAllBubbles(state) {
    const ide = window.conversationalIDE;
    if (!ide) return;
    
    document.querySelectorAll('.inline-content').forEach(contentEl => {
        const componentId = contentEl.id.replace('content-', '');
        
        contentEl.classList.remove('collapsed', 'compressed');
        if (state !== 'expand') {
            contentEl.classList.add(state === 'compress' ? 'compressed' : 'collapsed');
        }
        
        ide.bubbleStates.set(componentId, 
            state === 'expand' ? 'expanded' : 
            state === 'compress' ? 'compressed' : 'collapsed'
        );
    });
    
    ide.saveState();
}

function sendMessage() {
    window.conversationalIDE.sendMessage();
}

// Initialize on page load
window.addEventListener('DOMContentLoaded', () => {
    window.conversationalIDE = new ConversationalIDE();
});

// Save state before unload
window.addEventListener('beforeunload', () => {
    if (window.conversationalIDE) {
        window.conversationalIDE.saveState();
    }
});