#!/usr/bin/env node

/**
 * MCP Bridge for Android Playground
 * This script bridges between Claude/Gemini and the MCP server
 */

const http = require('http');
const https = require('https');
const readline = require('readline');
const { EventEmitter } = require('events');

const MCP_SERVER_URL = process.env.MCP_SERVER_URL || 'http://localhost:8080/mcp';
const SESSION_ID = process.env.SESSION_ID || `session-${Date.now()}`;

class MCPBridge extends EventEmitter {
    constructor() {
        super();
        this.sessionId = SESSION_ID;
        this.sseClient = null;
        this.rl = readline.createInterface({
            input: process.stdin,
            output: process.stdout
        });
    }

    async start() {
        console.error(`[MCP Bridge] Starting session: ${this.sessionId}`);
        
        // Connect to SSE endpoint
        await this.connectSSE();
        
        // Listen for stdin commands from Claude/Gemini
        this.rl.on('line', async (line) => {
            try {
                const message = JSON.parse(line);
                await this.handleMessage(message);
            } catch (error) {
                console.error('[MCP Bridge] Failed to parse message:', error);
                this.sendResponse({
                    type: 'error',
                    error: error.message
                });
            }
        });
        
        // Send ready signal
        this.sendResponse({
            type: 'ready',
            sessionId: this.sessionId,
            tools: await this.getAvailableTools()
        });
    }

    async connectSSE() {
        const url = `${MCP_SERVER_URL}/sse/${this.sessionId}`;
        console.error(`[MCP Bridge] Connecting to SSE: ${url}`);
        
        const protocol = url.startsWith('https') ? https : http;
        
        return new Promise((resolve, reject) => {
            protocol.get(url, (res) => {
                if (res.statusCode !== 200) {
                    reject(new Error(`SSE connection failed: ${res.statusCode}`));
                    return;
                }
                
                console.error('[MCP Bridge] SSE connected');
                this.sseClient = res;
                
                let buffer = '';
                res.on('data', (chunk) => {
                    buffer += chunk.toString();
                    
                    // Parse SSE events
                    const lines = buffer.split('\n');
                    buffer = lines.pop() || '';
                    
                    for (const line of lines) {
                        if (line.startsWith('data: ')) {
                            const data = line.slice(6);
                            if (data && data !== 'heartbeat') {
                                try {
                                    const event = JSON.parse(data);
                                    this.handleSSEEvent(event);
                                } catch (e) {
                                    console.error('[MCP Bridge] Failed to parse SSE event:', e);
                                }
                            }
                        }
                    }
                });
                
                res.on('error', (error) => {
                    console.error('[MCP Bridge] SSE error:', error);
                    this.reconnectSSE();
                });
                
                res.on('end', () => {
                    console.error('[MCP Bridge] SSE disconnected');
                    this.reconnectSSE();
                });
                
                resolve();
            }).on('error', reject);
        });
    }

    async reconnectSSE() {
        console.error('[MCP Bridge] Reconnecting SSE in 3 seconds...');
        setTimeout(() => {
            this.connectSSE().catch(console.error);
        }, 3000);
    }

    handleSSEEvent(event) {
        // Forward SSE events to stdout for Claude/Gemini
        this.sendResponse({
            type: 'sse_event',
            event: event
        });
    }

    async handleMessage(message) {
        console.error('[MCP Bridge] Handling message:', message.type);
        
        switch (message.type) {
            case 'tool_call':
                await this.handleToolCall(message);
                break;
            case 'prompt':
                await this.sendPrompt(message);
                break;
            case 'list_tools':
                const tools = await this.getAvailableTools();
                this.sendResponse({ type: 'tools', tools });
                break;
            default:
                console.error('[MCP Bridge] Unknown message type:', message.type);
        }
    }

    async handleToolCall(message) {
        const url = `${MCP_SERVER_URL}/message`;
        const payload = {
            session_id: this.sessionId,
            message: {
                type: 'ToolCall',
                id: message.id || `call-${Date.now()}`,
                tool: message.tool,
                arguments: message.arguments
            }
        };
        
        try {
            const response = await this.postJSON(url, payload);
            this.sendResponse({
                type: 'tool_result',
                id: message.id,
                result: response
            });
        } catch (error) {
            this.sendResponse({
                type: 'tool_error',
                id: message.id,
                error: error.message
            });
        }
    }

    async sendPrompt(message) {
        const url = `${MCP_SERVER_URL}/prompt`;
        const payload = {
            content: message.content,
            context_files: message.context_files,
            session_id: this.sessionId
        };
        
        try {
            const response = await this.postJSON(url, payload);
            this.sendResponse({
                type: 'prompt_sent',
                result: response
            });
        } catch (error) {
            this.sendResponse({
                type: 'error',
                error: error.message
            });
        }
    }

    async getAvailableTools() {
        const url = `${MCP_SERVER_URL}/tools`;
        
        return new Promise((resolve, reject) => {
            const protocol = url.startsWith('https') ? https : http;
            
            protocol.get(url, (res) => {
                let data = '';
                res.on('data', chunk => data += chunk);
                res.on('end', () => {
                    try {
                        const result = JSON.parse(data);
                        resolve(result.tools || []);
                    } catch (error) {
                        reject(error);
                    }
                });
            }).on('error', reject);
        });
    }

    async postJSON(url, data) {
        return new Promise((resolve, reject) => {
            const urlObj = new URL(url);
            const protocol = url.startsWith('https') ? https : http;
            
            const options = {
                hostname: urlObj.hostname,
                port: urlObj.port,
                path: urlObj.pathname,
                method: 'POST',
                headers: {
                    'Content-Type': 'application/json'
                }
            };
            
            const req = protocol.request(options, (res) => {
                let data = '';
                res.on('data', chunk => data += chunk);
                res.on('end', () => {
                    try {
                        resolve(JSON.parse(data));
                    } catch (error) {
                        resolve(data);
                    }
                });
            });
            
            req.on('error', reject);
            req.write(JSON.stringify(data));
            req.end();
        });
    }

    sendResponse(message) {
        // Send to stdout for Claude/Gemini to read
        console.log(JSON.stringify(message));
    }
}

// Start the bridge
const bridge = new MCPBridge();
bridge.start().catch(error => {
    console.error('[MCP Bridge] Failed to start:', error);
    process.exit(1);
});

// Handle graceful shutdown
process.on('SIGINT', () => {
    console.error('[MCP Bridge] Shutting down...');
    if (bridge.sseClient) {
        bridge.sseClient.destroy();
    }
    process.exit(0);
});