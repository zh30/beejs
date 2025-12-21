/**
 * 语言服务器基础类
 * 提供语言服务器的基本功能
 */

import {
    LanguageClient,
    LanguageClientOptions,
    ServerOptions,
    TransportKind
} from 'vscode-languageclient/node';
import * as vscode from 'vscode';

export abstract class LanguageServer {
    protected client?: LanguageClient;
    protected serverProcess?: any;
    protected readonly serverName: string;

    constructor(serverName: string) {
        this.serverName = serverName;
    }

    /**
     * 启动服务器
     */
    async start(context: vscode.ExtensionContext): Promise<void> {
        const serverModule = context.asAbsolutePath('out/language-server.js');

        const serverOptions: ServerOptions = {
            run: { module: serverModule, transport: TransportKind.ipc },
            debug: { module: serverModule, transport: TransportKind.ipc }
        };

        const clientOptions: LanguageClientOptions = {
            documentSelector: [
                { scheme: 'file', language: 'javascript' },
                { scheme: 'file', language: 'typescript' },
                { scheme: 'file', language: 'javascriptreact' },
                { scheme: 'file', language: 'typescriptreact' }
            ],
            synchronize: {
                fileEvents: vscode.workspace.createFileSystemWatcher('**/*.{js,ts,jsx,tsx}')
            }
        };

        this.client = new LanguageClient(this.serverName, this.serverName, serverOptions, clientOptions);

        this.onInitialize();

        try {
            await this.client.start();
            console.log(`${this.serverName} 已启动`);
        } catch (error) {
            console.error(`启动 ${this.serverName} 失败:`, error);
        }
    }

    /**
     * 停止服务器
     */
    async stop(): Promise<void> {
        if (this.client) {
            await this.client.stop();
            this.client = undefined;
            console.log(`${this.serverName} 已停止`);
        }
    }

    /**
     * 初始化回调
     */
    protected abstract onInitialize(): void;

    /**
     * 配置更改回调
     */
    protected onDidChangeConfiguration(): void {
        // 默认空实现
    }

    /**
     * 监听文件更改回调
     */
    protected onDidChangeWatchedFiles(): void {
        // 默认空实现
    }

    /**
     * 获取客户端
     */
    public getClient(): LanguageClient | undefined {
        return this.client;
    }
}
