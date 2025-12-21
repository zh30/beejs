/**
 * Beejs 脚本运行器
 * 负责执行 JavaScript/TypeScript 脚本
 */

import * as vscode from 'vscode';
import * as path from 'path';

export class BeejsRunner {
    private terminal?: vscode.Terminal;

    constructor() {
        this.terminal = undefined;
    }

    /**
     * 运行脚本
     */
    async runScript(uri?: vscode.Uri): Promise<void> {
        try {
            const filePath = uri?.fsPath || this.getActiveEditorFilePath();
            if (!filePath) {
                vscode.window.showErrorMessage('请打开一个 JavaScript 或 TypeScript 文件');
                return;
            }

            const config = vscode.workspace.getConfiguration('beejs');
            const runtimePath = config.get<string>('runtimePath') || 'beejs';
            const heapSize = config.get<number>('heapSize') || 512;

            // 创建终端或使用现有终端
            if (!this.terminal || this.terminal.exitCode !== undefined) {
                this.terminal = vscode.window.createTerminal('Beejs Runner');
            }

            // 设置环境变量
            const env = { ...process.env };
            env.BEEJS_HEAP_SIZE = heapSize.toString();

            // 执行命令
            const command = `"${runtimePath}" "${filePath}"`;
            this.terminal.sendText(command);
            this.terminal.show();

            // 显示状态消息
            vscode.window.setStatusBarMessage('$(play) 正在运行 Beejs 脚本...', 2000);

        } catch (error) {
            vscode.window.showErrorMessage(`运行脚本失败: ${error instanceof Error ? error.message : String(error)}`);
        }
    }

    /**
     * 调试脚本
     */
    async debugScript(uri?: vscode.Uri): Promise<void> {
        const filePath = uri?.fsPath || this.getActiveEditorFilePath();
        if (!filePath) {
            vscode.window.showErrorMessage('请打开一个 JavaScript 或 TypeScript 文件');
            return;
        }

        const config = vscode.workspace.getConfiguration('beejs');
        const runtimePath = config.get<string>('runtimePath') || 'beejs';

        // 启动调试会话
        const debugConfig: vscode.DebugConfiguration = {
            type: 'node',
            request: 'launch',
            name: 'Beejs Debug',
            program: filePath,
            runtimeExecutable: runtimePath,
            runtimeArgs: ['--inspect-brk'],
            console: 'integratedTerminal',
            internalConsoleOptions: 'neverOpen',
        };

        vscode.debug.startDebugging(vscode.workspace.workspaceFolders?.[0], debugConfig);
    }

    /**
     * 启动 REPL
     */
    async startRepl(): Promise<void> {
        const config = vscode.workspace.getConfiguration('beejs');
        const runtimePath = config.get<string>('runtimePath') || 'beejs';

        if (!this.terminal || this.terminal.exitCode !== undefined) {
            this.terminal = vscode.window.createTerminal('Beejs REPL');
        }

        this.terminal.sendText(`"${runtimePath}" --repl`);
        this.terminal.show();

        vscode.window.setStatusBarMessage('$(terminal) Beejs REPL 已启动', 2000);
    }

    /**
     * 执行代码片段
     */
    async evaluateCode(code: string): Promise<void> {
        const config = vscode.workspace.getConfiguration('beejs');
        const runtimePath = config.get<string>('runtimePath') || 'beejs';

        // 创建临时文件
        const tempFile = path.join(vscode.workspace.workspaceFolders?.[0]?.uri.fsPath || '', '.beejs_temp.js');
        require('fs').writeFileSync(tempFile, code);

        try {
            if (!this.terminal || this.terminal.exitCode !== undefined) {
                this.terminal = vscode.window.createTerminal('Beejs Runner');
            }

            this.terminal.sendText(`"${runtimePath}" "${tempFile}"`);
            this.terminal.show();

        } finally {
            // 清理临时文件
            try {
                require('fs').unlinkSync(tempFile);
            } catch (e) {
                // 忽略清理错误
            }
        }
    }

    /**
     * 运行性能测试
     */
    async runBenchmark(scriptPath?: string): Promise<void> {
        const filePath = scriptPath || this.getActiveEditorFilePath();
        if (!filePath) {
            vscode.window.showErrorMessage('请打开一个文件');
            return;
        }

        const config = vscode.workspace.getConfiguration('beejs');
        const runtimePath = config.get<string>('runtimePath') || 'beejs';

        if (!this.terminal || this.terminal.exitCode !== undefined) {
            this.terminal = vscode.window.createTerminal('Beejs Benchmark');
        }

        this.terminal.sendText(`"${runtimePath}" --benchmark "${filePath}"`);
        this.terminal.show();

        vscode.window.setStatusBarMessage('$(dashboard) 正在运行性能测试...', 2000);
    }

    /**
     * 获取当前编辑器中的文件路径
     */
    private getActiveEditorFilePath(): string | undefined {
        const editor = vscode.window.activeTextEditor;
        if (editor && this.isJavaScriptOrTypeScriptFile(editor.document)) {
            return editor.document.uri.fsPath;
        }
        return undefined;
    }

    /**
     * 检查是否是 JavaScript 或 TypeScript 文件
     */
    private isJavaScriptOrTypeScriptFile(document: vscode.TextDocument): boolean {
        const languageId = document.languageId;
        return languageId === 'javascript' ||
               languageId === 'typescript' ||
               languageId === 'javascriptreact' ||
               languageId === 'typescriptreact';
    }

    /**
     * 终止运行
     */
    terminate(): void {
        if (this.terminal) {
            this.terminal.dispose();
            this.terminal = undefined;
        }
    }
}
