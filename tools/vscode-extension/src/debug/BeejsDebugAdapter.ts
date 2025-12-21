/**
 * Beejs Debug Adapter
 *
 * Provides debugging capabilities for Beejs runtime:
 * - Launch and attach debugging
 * - Breakpoint management
 * - Stepping through code
 * - Variable inspection
 * - Call stack navigation
 */

import * as vscode from 'vscode';
import { DebugAdapterDescriptor, DebugAdapterDescriptorFactory, DebugAdapterInlineImplementation, DebugSession, DebugConfiguration } from 'vscode';
import { BeejsConfiguration } from '../utils/BeejsConfiguration';
import { spawn } from 'child_process';
import * as net from 'net';

export class BeejsDebugAdapterDescriptorFactory implements DebugAdapterDescriptorFactory {
    private config: BeejsConfiguration;

    constructor(config: BeejsConfiguration) {
        this.config = config;
    }

    public createDebugAdapterDescriptor(
        session: DebugSession,
        executable: vscode.DebugAdapterExecutable | undefined
    ): DebugAdapterDescriptor {
        // For inline implementation
        return new DebugAdapterInlineImplementation(new BeejsDebugAdapter());
    }
}

class BeejsDebugAdapter implements vscode.DebugAdapter {
    private socket: net.Socket | null = null;
    private server: net.Server | null = null;
    private messageQueue: string[] = [];
    private outputChannel: vscode.OutputChannel;

    constructor() {
        this.outputChannel = vscode.window.createOutputChannel('Beejs Debug');
    }

    public handleMessage(message: vscode.ProtocolMessage): void {
        this.outputChannel.appendLine(`Received: ${JSON.stringify(message, null, 2)}`);

        if (message.type === 'request') {
            this.handleRequest(message as vscode.Request);
        }
    }

    private handleRequest(request: vscode.Request): void {
        switch (request.command) {
            case 'initialize':
                this.sendResponse(request, {
                    body: {
                        supportsConfigurationDoneRequest: true,
                        supportsEvaluateForHovers: true,
                        supportsConditionalBreakpoints: true,
                        supportsHitConditionalBreakpoints: true,
                        supportsFunctionBreakpoints: true,
                        supportsExceptionBreakpoints: true,
                        supportsTerminateRequest: true,
                    },
                });
                break;

            case 'launch':
                this.handleLaunch(request);
                break;

            case 'attach':
                this.handleAttach(request);
                break;

            case 'disconnect':
                this.handleDisconnect();
                break;

            case 'terminate':
                this.handleTerminate();
                break;

            default:
                this.sendErrorResponse(request, {
                    id: 1,
                    format: 'Unknown command: {command}',
                    variables: { command: request.command },
                });
        }
    }

    private handleLaunch(request: vscode.Request): void {
        const args = request.arguments as DebugConfiguration & { program?: string };

        if (!args.program) {
            this.sendErrorResponse(request, {
                id: 1,
                format: 'No program specified',
            });
            return;
        }

        // Spawn Beejs process with debug port
        const beejsPath = vscode.workspace.getConfiguration('beejs').get('runtimePath', 'beejs');
        const debugPort = vscode.workspace.getConfiguration('beejs').get('debugPort', 9229);

        this.outputChannel.appendLine(`Launching: ${beejsPath} debug ${args.program}`);

        const child = spawn(beejsPath, ['debug', '--port', debugPort.toString(), args.program], {
            stdio: ['pipe', 'pipe', 'pipe'],
        });

        child.stdout.on('data', (data) => {
            this.outputChannel.append(`stdout: ${data}`);
        });

        child.stderr.on('data', (data) => {
            this.outputChannel.append(`stderr: ${data}`);
            // Parse debug messages from stderr
            this.parseDebugMessages(data.toString());
        });

        child.on('exit', (code) => {
            this.outputChannel.appendLine(`Process exited with code ${code}`);
            this.sendEvent(new vscode.TerminatedEvent());
        });

        // Connect to debug port
        this.connectToDebugger(debugPort);

        this.sendResponse(request);
    }

    private handleAttach(request: vscode.Request): void {
        const args = request.arguments as DebugConfiguration & { port?: number };
        const port = args.port || vscode.workspace.getConfiguration('beejs').get('debugPort', 9229);

        this.outputChannel.appendLine(`Attaching to Beejs on port ${port}`);
        this.connectToDebugger(port);

        this.sendResponse(request);
    }

    private connectToDebugger(port: number): void {
        this.server = net.createServer((socket) => {
            this.socket = socket;
            socket.on('data', (data) => {
                const message = data.toString();
                this.outputChannel.appendLine(`Debug message: ${message}`);

                try {
                    const event = JSON.parse(message);
                    this.sendEvent(event);
                } catch (e) {
                    this.outputChannel.appendLine(`Failed to parse debug message: ${e}`);
                }
            });

            socket.on('end', () => {
                this.outputChannel.appendLine('Disconnected from debug session');
            });
        });

        this.server.listen(port, () => {
            this.outputChannel.appendLine(`Debug server listening on port ${port}`);
        });
    }

    private parseDebugMessages(data: string): void {
        // Parse Beejs debug protocol messages
        // Format: DAP (Debug Adapter Protocol) compatible
        const lines = data.split('\n');

        for (const line of lines) {
            if (line.trim().startsWith('BP:')) {
                // Breakpoint hit
                const [, file, lineStr] = line.split(':');
                this.sendEvent(
                    new vscode.StoppedEvent('breakpoint', 'main', {
                        line: parseInt(lineStr, 10),
                        source: {
                            path: file,
                        },
                    })
                );
            } else if (line.trim().startsWith('EXC:')) {
                // Exception
                const [, message] = line.split(':', 2);
                this.sendEvent(new vscode.StoppedEvent('exception', 'main', { text: message }));
            }
        }
    }

    private handleDisconnect(): void {
        if (this.socket) {
            this.socket.end();
            this.socket = null;
        }
        if (this.server) {
            this.server.close();
            this.server = null;
        }
    }

    private handleTerminate(): void {
        this.handleDisconnect();
        this.sendEvent(new vscode.TerminatedEvent());
    }

    private sendResponse(request: vscode.Request, body?: any): void {
        const response: vscode.Response = {
            type: 'response',
            request_seq: request.seq,
            success: true,
            command: request.command,
            seq: this.getNextSeq(),
        };

        if (body) {
            response.body = body;
        }

        this.sendMessage(response);
    }

    private sendErrorResponse(request: vscode.Request, error: any): void {
        const response: vscode.Response = {
            type: 'response',
            request_seq: request.seq,
            success: false,
            command: request.command,
            seq: this.getNextSeq(),
            message: error.format,
        };

        if (error.variables) {
            response.body = { error: { format: error.format, variables: error.variables } };
        }

        this.sendMessage(response);
    }

    private sendEvent(event: vscode.Event): void {
        const dapEvent: vscode.Event = {
            type: 'event',
            event: event.event,
            seq: this.getNextSeq(),
            body: (event as any).body,
        };

        this.sendMessage(dapEvent);
    }

    private sendMessage(message: vscode.ProtocolMessage): void {
        if (this.socket && this.socket.writable) {
            this.socket.write(JSON.stringify(message) + '\n');
        } else {
            this.messageQueue.push(JSON.stringify(message));
        }
    }

    private seqCounter = 0;
    private getNextSeq(): number {
        return ++this.seqCounter;
    }

    public dispose(): void {
        this.handleDisconnect();
        this.outputChannel.dispose();
    }
}
