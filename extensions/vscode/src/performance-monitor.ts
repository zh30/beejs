/**
 * 性能监控器
 * 监控和显示运行时性能指标
 */

import * as vscode from 'vscode';
import * as path from 'path';

export interface PerformanceMetrics {
    executionTime: number;
    memoryUsage: number;
    heapSize: number;
    cpuUsage: number;
    timestamp: number;
}

export class PerformanceMonitor {
    private outputChannel: vscode.OutputChannel;
    private statusBarItem: vscode.StatusBarItem;
    private metricsHistory: PerformanceMetrics[] = [];
    private maxHistorySize: number = 100;

    constructor() {
        this.outputChannel = vscode.window.createOutputChannel('Beejs Performance');
        this.statusBarItem = vscode.window.createStatusBarItem(vscode.StatusBarAlignment.Right, 101);
        this.statusBarItem.command = 'beejs.showPerformanceReport';
        this.statusBarItem.text = '$(pulse)';
        this.statusBarItem.tooltip = '点击查看性能报告';
        this.statusBarItem.show();
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

        this.outputChannel.clear();
        this.outputChannel.show();
        this.outputChannel.appendLine('正在运行性能测试...');
        this.outputChannel.appendLine(`文件: ${filePath}`);
        this.outputChannel.appendLine('');

        try {
            const { spawn } = require('child_process');
            const child = spawn(`"${runtimePath}" --benchmark "${filePath}"`, { shell: true });

            child.stdout.on('data', (data: Buffer) => {
                this.outputChannel.append(data.toString());
            });

            child.stderr.on('data', (data: Buffer) => {
                this.outputChannel.append(`错误: ${data.toString()}`);
            });

            child.on('close', (code: number) => {
                if (code === 0) {
                    this.outputChannel.appendLine('');
                    this.outputChannel.appendLine('性能测试完成');
                } else {
                    this.outputChannel.appendLine('');
                    this.outputChannel.appendLine(`性能测试失败，退出代码: ${code}`);
                }
            });

        } catch (error) {
            vscode.window.showErrorMessage(`运行性能测试失败: ${error instanceof Error ? error.message : String(error)}`);
        }
    }

    /**
     * 显示性能报告
     */
    async showReport(): Promise<void> {
        const panel = vscode.window.createWebviewPanel(
            'beejsPerformanceReport',
            'Beejs 性能报告',
            vscode.ViewColumn.One,
            {
                enableScripts: true,
                localResourceRoots: [vscode.Uri.joinPath(vscode.extensionUri, 'media')]
            }
        );

        const reportHtml = this.generateReportHtml();
        panel.webview.html = reportHtml;
    }

    /**
     * 更新指标
     */
    async updateMetrics(document: vscode.TextDocument): Promise<void> {
        // 模拟性能指标
        const metrics: PerformanceMetrics = {
            executionTime: Math.random() * 100,
            memoryUsage: Math.random() * 50,
            heapSize: Math.random() * 100,
            cpuUsage: Math.random() * 20,
            timestamp: Date.now()
        };

        this.addMetrics(metrics);

        // 更新状态栏
        this.updateStatusBar(metrics);
    }

    /**
     * 更新视图
     */
    async updateView(document: vscode.TextDocument): Promise<void> {
        if (document.languageId.startsWith('javascript') || document.languageId.startsWith('typescript')) {
            await this.updateMetrics(document);
        }
    }

    /**
     * 添加性能指标
     */
    private addMetrics(metrics: PerformanceMetrics): void {
        this.metricsHistory.push(metrics);
        if (this.metricsHistory.length > this.maxHistorySize) {
            this.metricsHistory.shift();
        }
    }

    /**
     * 更新状态栏
     */
    private updateStatusBar(metrics: PerformanceMetrics): void {
        const icon = this.getPerformanceIcon(metrics);
        this.statusBarItem.text = icon;
        this.statusBarItem.tooltip = `执行时间: ${metrics.executionTime.toFixed(2)}ms\n内存使用: ${metrics.memoryUsage.toFixed(2)}MB`;
    }

    /**
     * 获取性能图标
     */
    private getPerformanceIcon(metrics: PerformanceMetrics): string {
        if (metrics.executionTime > 50) {
            return '$(alert)';
        } else if (metrics.executionTime > 20) {
            return '$(warning)';
        } else {
            return '$(check)';
        }
    }

    /**
     * 生成报告 HTML
     */
    private generateReportHtml(): string {
        const recentMetrics = this.metricsHistory.slice(-10);
        const avgExecutionTime = recentMetrics.reduce((sum, m) => sum + m.executionTime, 0) / recentMetrics.length;
        const avgMemoryUsage = recentMetrics.reduce((sum, m) => sum + m.memoryUsage, 0) / recentMetrics.length;

        return `
        <!DOCTYPE html>
        <html>
        <head>
            <meta charset="UTF-8">
            <meta name="viewport" content="width=device-width, initial-scale=1.0">
            <title>Beejs 性能报告</title>
            <style>
                body { font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif; padding: 20px; }
                h1 { color: #007acc; }
                .metric { background: #f0f0f0; padding: 15px; margin: 10px 0; border-radius: 5px; }
                .metric-value { font-size: 24px; font-weight: bold; color: #333; }
                .metric-label { color: #666; font-size: 14px; }
                table { width: 100%; border-collapse: collapse; margin-top: 20px; }
                th, td { padding: 8px; text-align: left; border-bottom: 1px solid #ddd; }
                th { background-color: #007acc; color: white; }
                .good { color: #28a745; }
                .warning { color: #ffc107; }
                .error { color: #dc3545; }
            </style>
        </head>
        <body>
            <h1>🚀 Beejs 性能报告</h1>

            <div class="metric">
                <div class="metric-label">平均执行时间</div>
                <div class="metric-value ${avgExecutionTime > 50 ? 'error' : avgExecutionTime > 20 ? 'warning' : 'good'}">${avgExecutionTime.toFixed(2)} ms</div>
            </div>

            <div class="metric">
                <div class="metric-label">平均内存使用</div>
                <div class="metric-value">${avgMemoryUsage.toFixed(2)} MB</div>
            </div>

            <div class="metric">
                <div class="metric-label">测量次数</div>
                <div class="metric-value">${this.metricsHistory.length}</div>
            </div>

            <h2>最近性能数据</h2>
            <table>
                <thead>
                    <tr>
                        <th>时间</th>
                        <th>执行时间 (ms)</th>
                        <th>内存 (MB)</th>
                        <th>堆大小 (MB)</th>
                        <th>CPU (%)</th>
                    </tr>
                </thead>
                <tbody>
                    ${recentMetrics.map(m => `
                        <tr>
                            <td>${new Date(m.timestamp).toLocaleTimeString()}</td>
                            <td class="${m.executionTime > 50 ? 'error' : m.executionTime > 20 ? 'warning' : 'good'}">${m.executionTime.toFixed(2)}</td>
                            <td>${m.memoryUsage.toFixed(2)}</td>
                            <td>${m.heapSize.toFixed(2)}</td>
                            <td>${m.cpuUsage.toFixed(2)}</td>
                        </tr>
                    `).join('')}
                </tbody>
            </table>
        </body>
        </html>
        `;
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
     * 清理资源
     */
    dispose(): void {
        this.statusBarItem.dispose();
        this.outputChannel.dispose();
    }
}
