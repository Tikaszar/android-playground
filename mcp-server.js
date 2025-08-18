#!/usr/bin/env node

/**
 * MCP Server for Android Playground
 * Implements JSON-RPC 2.0 protocol over stdio for Claude Code
 */

const readline = require('readline');
const http = require('http');

// MCP Server URL
const PLAYGROUND_SERVER = process.env.MCP_SERVER_URL || 'http://localhost:8080/mcp';
const SESSION_ID = `session-${Date.now()}`;

// JSON-RPC 2.0 helper functions
function sendResponse(id, result) {
    const response = {
        jsonrpc: "2.0",
        id: id,
        result: result
    };
    console.log(JSON.stringify(response));
}

function sendError(id, code, message, data = null) {
    const response = {
        jsonrpc: "2.0",
        id: id,
        error: {
            code: code,
            message: message,
            data: data
        }
    };
    console.log(JSON.stringify(response));
}

function sendNotification(method, params) {
    const notification = {
        jsonrpc: "2.0",
        method: method,
        params: params
    };
    console.log(JSON.stringify(notification));
}

// HTTP helper for communicating with playground server
async function httpRequest(method, path, data = null) {
    return new Promise((resolve, reject) => {
        const url = new URL(PLAYGROUND_SERVER + path);
        const options = {
            hostname: url.hostname,
            port: url.port || 80,
            path: url.pathname,
            method: method,
            headers: {
                'Content-Type': 'application/json'
            }
        };

        const req = http.request(options, (res) => {
            let body = '';
            res.on('data', chunk => body += chunk);
            res.on('end', () => {
                try {
                    resolve(JSON.parse(body));
                } catch (e) {
                    resolve(body);
                }
            });
        });

        req.on('error', reject);
        if (data) {
            req.write(JSON.stringify(data));
        }
        req.end();
    });
}

// Available tools mapping
const TOOLS = {
    show_file: {
        description: "Display file content in the browser editor",
        inputSchema: {
            type: "object",
            properties: {
                path: { type: "string", description: "File path to display" },
                content: { type: "string", description: "File content" }
            },
            required: ["path", "content"]
        }
    },
    update_editor: {
        description: "Update the current editor content",
        inputSchema: {
            type: "object",
            properties: {
                content: { type: "string", description: "New editor content" }
            },
            required: ["content"]
        }
    },
    show_terminal_output: {
        description: "Display output in the terminal",
        inputSchema: {
            type: "object",
            properties: {
                output: { type: "string", description: "Terminal output to display" }
            },
            required: ["output"]
        }
    },
    update_file_tree: {
        description: "Update the file browser tree",
        inputSchema: {
            type: "object",
            properties: {
                files: { 
                    type: "array", 
                    description: "File tree structure",
                    items: { type: "object" }
                }
            },
            required: ["files"]
        }
    },
    show_diff: {
        description: "Display a diff view",
        inputSchema: {
            type: "object",
            properties: {
                old_content: { type: "string", description: "Original content" },
                new_content: { type: "string", description: "Modified content" },
                filename: { type: "string", description: "File name for the diff" }
            },
            required: ["old_content", "new_content"]
        }
    },
    show_error: {
        description: "Show error message with location",
        inputSchema: {
            type: "object",
            properties: {
                message: { type: "string", description: "Error message" },
                file: { type: "string", description: "File where error occurred" },
                line: { type: "number", description: "Line number" },
                column: { type: "number", description: "Column number" }
            },
            required: ["message"]
        }
    },
    update_status_bar: {
        description: "Update status bar message",
        inputSchema: {
            type: "object",
            properties: {
                message: { type: "string", description: "Status message" }
            },
            required: ["message"]
        }
    },
    show_notification: {
        description: "Display a notification",
        inputSchema: {
            type: "object",
            properties: {
                message: { type: "string", description: "Notification message" },
                type: { 
                    type: "string", 
                    enum: ["info", "warning", "error", "success"],
                    description: "Notification type" 
                }
            },
            required: ["message"]
        }
    },
    open_panel: {
        description: "Open a specific IDE panel",
        inputSchema: {
            type: "object",
            properties: {
                panel: { 
                    type: "string",
                    enum: ["editor", "terminal", "files", "chat", "debugger"],
                    description: "Panel to open" 
                }
            },
            required: ["panel"]
        }
    },
    show_chat_message: {
        description: "Display message in conversation",
        inputSchema: {
            type: "object",
            properties: {
                message: { type: "string", description: "Chat message" },
                sender: { 
                    type: "string",
                    enum: ["user", "assistant"],
                    description: "Message sender" 
                }
            },
            required: ["message", "sender"]
        }
    }
};

// Handle JSON-RPC requests
async function handleRequest(request) {
    const { id, method, params } = request;

    try {
        switch (method) {
            case 'initialize':
                // Initialize session with playground server
                await httpRequest('POST', '/message', {
                    session_id: SESSION_ID,
                    message: { type: 'Connect' }
                });
                
                sendResponse(id, {
                    protocolVersion: "1.0",
                    serverInfo: {
                        name: "android-playground",
                        version: "1.0.0"
                    },
                    capabilities: {
                        tools: true
                    }
                });
                break;

            case 'tools/list':
                // Return available tools
                sendResponse(id, {
                    tools: Object.entries(TOOLS).map(([name, tool]) => ({
                        name: name,
                        description: tool.description,
                        inputSchema: tool.inputSchema
                    }))
                });
                break;

            case 'tools/call':
                // Execute tool call
                const { name, arguments: args } = params;
                
                if (!TOOLS[name]) {
                    sendError(id, -32602, `Unknown tool: ${name}`);
                    return;
                }

                // Send tool call to playground server
                const result = await httpRequest('POST', '/message', {
                    session_id: SESSION_ID,
                    message: {
                        type: 'ToolCall',
                        id: `call-${Date.now()}`,
                        tool: name,
                        arguments: args
                    }
                });

                sendResponse(id, {
                    content: [
                        {
                            type: "text",
                            text: result.success ? `Tool ${name} executed successfully` : `Tool ${name} failed: ${result.error}`
                        }
                    ]
                });
                break;

            case 'completion/complete':
                // Handle completion requests if needed
                sendResponse(id, { completion: null });
                break;

            default:
                sendError(id, -32601, `Method not found: ${method}`);
        }
    } catch (error) {
        sendError(id, -32603, `Internal error: ${error.message}`);
    }
}

// Main server loop
const rl = readline.createInterface({
    input: process.stdin,
    output: process.stdout,
    terminal: false
});

// Log to stderr to avoid interfering with JSON-RPC
console.error('[MCP Server] Starting Android Playground MCP Server...');
console.error(`[MCP Server] Connecting to: ${PLAYGROUND_SERVER}`);
console.error(`[MCP Server] Session ID: ${SESSION_ID}`);

rl.on('line', async (line) => {
    try {
        const request = JSON.parse(line);
        if (request.jsonrpc === "2.0") {
            await handleRequest(request);
        }
    } catch (error) {
        console.error('[MCP Server] Error handling request:', error);
    }
});

// Handle shutdown gracefully
process.on('SIGINT', () => {
    console.error('[MCP Server] Shutting down...');
    process.exit(0);
});

process.on('SIGTERM', () => {
    console.error('[MCP Server] Shutting down...');
    process.exit(0);
});