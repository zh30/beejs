/**
 * 类型定义生成器
 * 自动生成 TypeScript 类型定义
 */

import * as vscode from 'vscode';
import * as path from 'path';
import { spawn } from 'child_process';

export class TypeGenerator {
    private outputChannel: vscode.OutputChannel;

    constructor() {
        this.outputChannel = vscode.window.createOutputChannel('Beejs Type Generation');
    }

    /**
     * 生成类型定义
     */
    async generateTypes(): Promise<void> {
        const workspaceFolder = vscode.workspace.workspaceFolders?.[0];
        if (!workspaceFolder) {
            vscode.window.showErrorMessage('请在打开的工作区中执行此命令');
            return;
        }

        const config = vscode.workspace.getConfiguration('beejs');
        const runtimePath = config.get<string>('runtimePath') || 'beejs';

        try {
            this.outputChannel.clear();
            this.outputChannel.show();
            this.outputChannel.appendLine('正在生成类型定义...');

            const command = `"${runtimePath}" --generate-types`;
            const child = spawn(command, { shell: true, cwd: workspaceFolder.uri.fsPath });

            child.stdout.on('data', (data) => {
                this.outputChannel.append(data.toString());
            });

            child.stderr.on('data', (data) => {
                this.outputChannel.append(`错误: ${data.toString()}`);
            });

            child.on('close', (code) => {
                if (code === 0) {
                    vscode.window.showInformationMessage('类型定义生成成功');
                    this.outputChannel.appendLine('类型定义生成完成');

                    // 刷新文件资源管理器
                    vscode.commands.executeCommand('workbench.files.action.refreshFilesExplorer');
                } else {
                    vscode.window.showErrorMessage(`类型定义生成失败，退出代码: ${code}`);
                }
            });

        } catch (error) {
            vscode.window.showErrorMessage(`生成类型定义时出错: ${error instanceof Error ? error.message : String(error)}`);
        }
    }

    /**
     * 为特定文件生成类型
     */
    async generateTypesForFile(uri: vscode.Uri): Promise<void> {
        const config = vscode.workspace.getConfiguration('beejs');
        const runtimePath = config.get<string>('runtimePath') || 'beejs';

        try {
            const command = `"${runtimePath}" --generate-types "${uri.fsPath}"`;
            const child = spawn(command, { shell: true });

            child.on('close', (code) => {
                if (code === 0) {
                    vscode.window.setStatusBarMessage('$(check) 类型生成完成', 2000);
                }
            });

        } catch (error) {
            console.error('生成类型定义失败:', error);
        }
    }

    /**
     * 监视文件变化并自动生成类型
     */
    setupWatcher(): vscode.FileSystemWatcher {
        const workspaceFolder = vscode.workspace.workspaceFolders?.[0];
        if (!workspaceFolder) {
            throw new Error('工作区未打开');
        }

        const watcher = vscode.workspace.createFileSystemWatcher('**/*.{js,jsx}');
        const config = vscode.workspace.getConfiguration('beejs');
        const enableTypeGeneration = config.get<boolean>('enableTypeGeneration');

        if (enableTypeGeneration) {
            watcher.onDidSave(async (e) => {
                if (e.scheme === 'file') {
                    await this.generateTypesForFile(e.uri);
                }
            });
        }

        return watcher;
    }

    /**
     * 生成项目范围的类型定义
     */
    async generateProjectTypes(): Promise<void> {
        const workspaceFolder = vscode.workspace.workspaceFolders?.[0];
        if (!workspaceFolder) {
            vscode.window.showErrorMessage('请在打开的工作区中执行此命令');
            return;
        }

        const config = vscode.workspace.getConfiguration('beejs');
        const runtimePath = config.get<string>('runtimePath') || 'beejs';

        // 选择输出目录
        const outputDir = await vscode.window.showInputBox({
            prompt: '类型定义输出目录',
            value: 'types'
        });

        if (!outputDir) {
            return;
        }

        try {
            this.outputChannel.clear();
            this.outputChannel.show();
            this.outputChannel.appendLine(`正在生成项目类型定义到 ${outputDir}...`);

            const command = `"${runtimePath}" --generate-project-types --output-dir="${outputDir}"`;
            const child = spawn(command, { shell: true, cwd: workspaceFolder.uri.fsPath });

            child.stdout.on('data', (data) => {
                this.outputChannel.append(data.toString());
            });

            child.stderr.on('data', (data) => {
                this.outputChannel.append(`错误: ${data.toString()}`);
            });

            child.on('close', (code) => {
                if (code === 0) {
                    vscode.window.showInformationMessage('项目类型定义生成成功');
                    this.outputChannel.appendLine('项目类型定义生成完成');

                    // 询问是否打开输出目录
                    const openDir = await vscode.window.showInformationMessage(
                        '类型定义生成完成，是否打开输出目录？',
                        '是',
                        '否'
                    );

                    if (openDir === '是') {
                        const fullOutputPath = path.join(workspaceFolder.uri.fsPath, outputDir);
                        vscode.commands.executeCommand('vscode.openFolder', vscode.Uri.file(fullOutputPath));
                    }
                } else {
                    vscode.window.showErrorMessage(`项目类型定义生成失败，退出代码: ${code}`);
                }
            });

        } catch (error) {
            vscode.window.showErrorMessage(`生成项目类型定义时出错: ${error instanceof Error ? error.message : String(error)}`);
        }
    }
}
