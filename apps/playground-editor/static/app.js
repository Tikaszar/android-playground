// Conversational IDE Client
class ConversationalIDE {
    constructor() {
        this.ws = null;
        this.currentChannel = 'general';
        this.messages = new Map();
        this.agents = new Map();
        this.reconnectAttempts = 0;
        this.maxReconnectAttempts = 10;
        this.reconnectDelay = 1000;
        
        this.init();
    }

    init() {
        this.connectWebSocket();
        this.setupEventListeners();
        this.loadInitialState();
    }

    connectWebSocket() {
        const wsUrl = 'ws://localhost:8080/ws';
        console.log('Connecting to WebSocket:', wsUrl);
        
        this.ws = new WebSocket(wsUrl);
        
        this.ws.onopen = () => {
            console.log('WebSocket connected');
            this.updateConnectionStatus(true);
            this.reconnectAttempts = 0;
            
            // Register for UI Framework Plugin channels (1200-1209)
            this.sendPacket(0, 1, { 
                type: 'register',
                name: 'conversational-ide-client',
                channels: [1200, 1201, 1202, 1203, 1204, 1205, 1206, 1207, 1208, 1209]
            });
        };

        this.ws.onmessage = (event) => {
            if (event.data instanceof Blob) {
                this.handleBinaryMessage(event.data);
            } else {
                this.handleTextMessage(JSON.parse(event.data));
            }
        };

        this.ws.onerror = (error) => {
            console.error('WebSocket error:', error);
        };

        this.ws.onclose = () => {
            console.log('WebSocket disconnected');
            this.updateConnectionStatus(false);
            this.attemptReconnect();
        };
    }

    attemptReconnect() {
        if (this.reconnectAttempts < this.maxReconnectAttempts) {
            this.reconnectAttempts++;
            console.log(`Reconnecting... (attempt ${this.reconnectAttempts})`);
            setTimeout(() => this.connectWebSocket(), this.reconnectDelay * this.reconnectAttempts);
        }
    }

    sendPacket(channelId, packetType, payload) {
        if (this.ws && this.ws.readyState === WebSocket.OPEN) {
            const packet = {
                channel_id: channelId,
                packet_type: packetType,
                priority: 2, // Medium priority
                payload: JSON.stringify(payload)
            };
            this.ws.send(JSON.stringify(packet));
        }
    }

    handleBinaryMessage(blob) {
        // Convert blob to array buffer and parse
        blob.arrayBuffer().then(buffer => {
            const view = new DataView(buffer);
            const channelId = view.getUint16(0, true);
            const packetType = view.getUint16(2, true);
            const priority = view.getUint8(4);
            const payloadSize = view.getUint32(5, true);
            
            const decoder = new TextDecoder();
            const payloadBytes = new Uint8Array(buffer, 9, payloadSize);
            const payload = JSON.parse(decoder.decode(payloadBytes));
            
            this.handlePacket(channelId, packetType, payload);
        });
    }

    handleTextMessage(data) {
        if (data.channel_id >= 1200 && data.channel_id <= 1209) {
            const payload = typeof data.payload === 'string' 
                ? JSON.parse(data.payload) 
                : data.payload;
            this.handlePacket(data.channel_id, data.packet_type, payload);
        }
    }

    handlePacket(channelId, packetType, payload) {
        console.log('Received packet:', { channelId, packetType, payload });
        
        // Handle different message types from UI Framework Plugin
        switch (payload.type) {
            case 'message':
                this.addMessage(payload);
                break;
            case 'channel_update':
                this.updateChannel(payload);
                break;
            case 'agent_status':
                this.updateAgentStatus(payload);
                break;
            case 'task_update':
                this.updateTaskQueue(payload);
                break;
            case 'file_update':
                this.updateActiveFiles(payload);
                break;
            case 'inline_component':
                this.handleInlineComponent(payload);
                break;
        }
    }

    addMessage(data) {
        const messagesDiv = document.getElementById('messages');
        const messageEl = document.createElement('div');
        messageEl.className = 'message';
        messageEl.dataset.messageId = data.id || Date.now();
        
        messageEl.innerHTML = `
            <div class="message-header">
                <span class="message-author">${data.author || 'System'}</span>
                <span class="message-time">${new Date(data.timestamp || Date.now()).toLocaleTimeString()}</span>
            </div>
            <div class="message-content">${this.formatContent(data.content)}</div>
        `;
        
        // Add inline components if present
        if (data.components) {
            data.components.forEach(comp => {
                messageEl.appendChild(this.createInlineComponent(comp));
            });
        }
        
        messagesDiv.appendChild(messageEl);
        messagesDiv.scrollTop = messagesDiv.scrollHeight;
    }

    createInlineComponent(component) {
        const compEl = document.createElement('div');
        compEl.className = 'inline-component';
        compEl.dataset.state = component.state || 'expanded';
        
        const header = document.createElement('div');
        header.className = 'component-header';
        header.innerHTML = `
            <span class="component-title">${component.title || 'Component'}</span>
            <span class="component-toggle">${this.getStateIcon(component.state)}</span>
        `;
        
        const content = document.createElement('div');
        content.className = `component-content ${component.state || 'expanded'}`;
        
        switch (component.type) {
            case 'editor':
                content.innerHTML = `<div class="inline-editor"><pre><code>${this.escapeHtml(component.content || '')}</code></pre></div>`;
                break;
            case 'file-browser':
                content.innerHTML = this.renderFileBrowser(component.files || []);
                break;
            case 'terminal':
                content.innerHTML = `<div class="inline-terminal">${this.escapeHtml(component.output || '')}</div>`;
                break;
            case 'diff':
                content.innerHTML = this.renderDiff(component.diff || {});
                break;
            default:
                content.innerHTML = `<div>${this.escapeHtml(component.content || '')}</div>`;
        }
        
        header.addEventListener('click', () => this.toggleComponentState(compEl));
        
        compEl.appendChild(header);
        compEl.appendChild(content);
        
        return compEl;
    }

    toggleComponentState(element) {
        const content = element.querySelector('.component-content');
        const toggle = element.querySelector('.component-toggle');
        const currentState = element.dataset.state;
        
        let newState;
        switch (currentState) {
            case 'collapsed':
                newState = 'compressed';
                break;
            case 'compressed':
                newState = 'expanded';
                break;
            case 'expanded':
                newState = 'collapsed';
                break;
            default:
                newState = 'expanded';
        }
        
        element.dataset.state = newState;
        content.className = `component-content ${newState}`;
        toggle.textContent = this.getStateIcon(newState);
    }

    getStateIcon(state) {
        switch (state) {
            case 'collapsed': return '▶';
            case 'compressed': return '▼';
            case 'expanded': return '▽';
            default: return '▽';
        }
    }

    formatContent(content) {
        // Convert markdown-like formatting
        return this.escapeHtml(content)
            .replace(/`([^`]+)`/g, '<code>$1</code>')
            .replace(/\*\*([^*]+)\*\*/g, '<strong>$1</strong>')
            .replace(/\*([^*]+)\*/g, '<em>$1</em>')
            .replace(/\n/g, '<br>');
    }

    escapeHtml(text) {
        const div = document.createElement('div');
        div.textContent = text;
        return div.innerHTML;
    }

    renderFileBrowser(files) {
        return `<div class="file-tree">${files.map(f => 
            `<div class="file-item">${f.name}</div>`
        ).join('')}</div>`;
    }

    renderDiff(diff) {
        return `<div class="diff-view">
            <pre>${this.escapeHtml(diff.content || 'No diff available')}</pre>
        </div>`;
    }

    updateConnectionStatus(connected) {
        const indicator = document.getElementById('connection-status');
        const text = document.getElementById('connection-text');
        
        if (connected) {
            indicator.className = 'status-indicator connected';
            text.textContent = 'Connected';
        } else {
            indicator.className = 'status-indicator disconnected';
            text.textContent = 'Disconnected';
        }
    }

    updateAgentStatus(data) {
        const agentEl = document.querySelector(`[data-agent="${data.agent}"]`);
        if (agentEl) {
            const statusEl = agentEl.querySelector('.agent-status');
            if (statusEl) {
                statusEl.className = `agent-status ${data.status}`;
            }
        }
    }

    updateTaskQueue(data) {
        const taskQueue = document.getElementById('task-queue');
        taskQueue.innerHTML = data.tasks.map(task => 
            `<div class="task-item">${task.title} - ${task.status}</div>`
        ).join('');
    }

    updateActiveFiles(data) {
        const fileList = document.getElementById('active-files');
        fileList.innerHTML = data.files.map(file => 
            `<div class="file-item">${file.name}</div>`
        ).join('');
    }

    setupEventListeners() {
        // Send message
        const sendButton = document.getElementById('send-button');
        const messageInput = document.getElementById('message-input');
        
        const sendMessage = () => {
            const content = messageInput.value.trim();
            if (content) {
                this.sendPacket(1200, 1, {
                    type: 'user_message',
                    channel: this.currentChannel,
                    content: content
                });
                
                // Add message locally
                this.addMessage({
                    author: 'You',
                    content: content,
                    timestamp: Date.now()
                });
                
                messageInput.value = '';
            }
        };
        
        sendButton.addEventListener('click', sendMessage);
        messageInput.addEventListener('keydown', (e) => {
            if (e.key === 'Enter' && !e.shiftKey) {
                e.preventDefault();
                sendMessage();
            }
        });
        
        // Channel switching
        document.querySelectorAll('.channel').forEach(ch => {
            ch.addEventListener('click', () => {
                document.querySelector('.channel.active')?.classList.remove('active');
                ch.classList.add('active');
                this.currentChannel = ch.dataset.channel;
                document.getElementById('current-channel').textContent = `# ${this.currentChannel}`;
                this.loadChannelMessages(this.currentChannel);
            });
        });
        
        // Bubble state controls
        document.getElementById('expand-all').addEventListener('click', () => {
            document.querySelectorAll('.inline-component').forEach(el => {
                el.dataset.state = 'expanded';
                el.querySelector('.component-content').className = 'component-content expanded';
                el.querySelector('.component-toggle').textContent = '▽';
            });
        });
        
        document.getElementById('compress-all').addEventListener('click', () => {
            document.querySelectorAll('.inline-component').forEach(el => {
                el.dataset.state = 'compressed';
                el.querySelector('.component-content').className = 'component-content compressed';
                el.querySelector('.component-toggle').textContent = '▼';
            });
        });
        
        document.getElementById('collapse-all').addEventListener('click', () => {
            document.querySelectorAll('.inline-component').forEach(el => {
                el.dataset.state = 'collapsed';
                el.querySelector('.component-content').className = 'component-content collapsed';
                el.querySelector('.component-toggle').textContent = '▶';
            });
        });
    }

    loadInitialState() {
        // Add welcome message
        this.addMessage({
            author: 'System',
            content: 'Welcome to the Conversational IDE! Type a message to start collaborating with AI agents.',
            timestamp: Date.now()
        });
    }

    loadChannelMessages(channel) {
        // Clear current messages
        document.getElementById('messages').innerHTML = '';
        
        // Request channel history from server
        this.sendPacket(1200, 2, {
            type: 'load_channel',
            channel: channel
        });
    }
}

// Initialize the IDE when DOM is ready
document.addEventListener('DOMContentLoaded', () => {
    window.ide = new ConversationalIDE();
});