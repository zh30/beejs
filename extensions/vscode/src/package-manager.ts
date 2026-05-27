/**
 * 包管理器集成
 * 支持 npm、Yarn、pnpm
 */

import * as vscode from 'vscode';
import * as path from 'path';
import { spawn } from 'child_process';

export class PackageManager {
    private terminal?: vscode.Terminal;

    /**
     * 安装依赖
     */
    async installDependencies(): Promise<void> {
        const workspaceFolder = vscode.workspace.workspaceFolders?.[0];
        if (!workspaceFolder) {
            vscode.window.showErrorMessage('请在打开的工作区中执行此命令');
            return;
        }

        const packageJsonPath = path.join(workspaceFolder.uri.fsPath, 'package.json');
        const fs = require('fs');

        if (!fs.existsSync(packageJsonPath)) {
            // 初始化项目
            await this.initializeProject();
        }

        const config = vscode.workspace.getConfiguration('beejs');
        const packageManager = config.get<string>('packageManager') || 'npm';
        const autoInstall = config.get<boolean>('enableAutoInstall');

        if (!this.terminal || this.terminal.exitCode !== undefined) {
            this.terminal = vscode.window.createTerminal('Package Manager');
        }

        let command: string;
        switch (packageManager) {
            case 'yarn':
                command = await this.installWithYarn(workspaceFolder.uri.fsPath);
                break;
            case 'pnpm':
                command = await this.installWithPnpm(workspaceFolder.uri.fsPath);
                break;
            default:
                command = await this.installWithNpm(workspaceFolder.uri.fsPath);
                break;
        }

        this.terminal.sendText(command);
        this.terminal.show();

        vscode.window.setStatusBarMessage('$(cloud-download) 正在安装依赖...', 3000);
    }

    /**
     * 使用 npm 安装
     */
    private async installWithNpm(workspacePath: string): Promise<string> {
        return 'npm install';
    }

    /**
     * 使用 Yarn 安装
     */
    private async installWithYarn(workspacePath: string): Promise<string> {
        return 'yarn install';
    }

    /**
     * 使用 pnpm 安装
     */
    private async installWithPnpm(workspacePath: string): Promise<string> {
        return 'pnpm install';
    }

    /**
     * 添加包
     */
    async addPackage(packageName: string, isDev: boolean = false): Promise<void> {
        const config = vscode.workspace.getConfiguration('beejs');
        const packageManager = config.get<string>('packageManager') || 'npm';

        if (!this.terminal || this.terminal.exitCode !== undefined) {
            this.terminal = vscode.window.createTerminal('Package Manager');
        }

        let command: string;
        switch (packageManager) {
            case 'yarn':
                command = `yarn add ${isDev ? '--dev ' : ''}${packageName}`;
                break;
            case 'pnpm':
                command = `pnpm add ${isDev ? '--save-dev ' : ''}${packageName}`;
                break;
            default:
                command = `npm install ${isDev ? '--save-dev ' : ''}${packageName}`;
                break;
        }

        this.terminal.sendText(command);
        this.terminal.show();
    }

    /**
     * 初始化项目
     */
    private async initializeProject(): Promise<void> {
        const projectName = await vscode.window.showInputBox({
            prompt: '项目名称',
            value: path.basename(vscode.workspace.workspaceFolders?.[0]?.uri.fsPath || 'my-project')
        });

        if (!projectName) {
            return;
        }

        const packageJson = {
            name: projectName,
            version: '1.0.0',
            description: '',
            main: 'index.js',
            scripts: {
                start: 'bee run index.js',
                test: 'bee test test.js'
            },
            keywords: [],
            author: '',
            license: 'ISC'
        };

        const workspacePath = vscode.workspace.workspaceFolders?.[0]?.uri.fsPath;
        if (workspacePath) {
            const fs = require('fs');
            const packageJsonPath = path.join(workspacePath, 'package.json');
            fs.writeFileSync(packageJsonPath, JSON.stringify(packageJson, null, 2));
        }
    }

    /**
     * 检测包管理器类型
     */
    detectPackageManager(workspacePath: string): 'npm' | 'yarn' | 'pnpm' {
        const fs = require('fs');

        if (fs.existsSync(path.join(workspacePath, 'pnpm-lock.yaml'))) {
            return 'pnpm';
        }
        if (fs.existsSync(path.join(workspacePath, 'yarn.lock'))) {
            return 'yarn';
        }
        return 'npm';
    }

    /**
     * 更新依赖
     */
    async updateDependencies(): Promise<void> {
        if (!this.terminal || this.terminal.exitCode !== undefined) {
            this.terminal = vscode.window.createTerminal('Package Manager');
        }

        const config = vscode.workspace.getConfiguration('beejs');
        const packageManager = config.get<string>('packageManager') || 'npm';

        let command: string;
        switch (packageManager) {
            case 'yarn':
                command = 'yarn upgrade';
                break;
            case 'pnpm':
                command = 'pnpm update';
                break;
            default:
                command = 'npm update';
                break;
        }

        this.terminal.sendText(command);
        this.terminal.show();

        vscode.window.setStatusBarMessage('$(cloud-download) 正在更新依赖...', 3000);
    }
}
