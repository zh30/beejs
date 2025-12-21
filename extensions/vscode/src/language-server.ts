/**
 * Beejs 语言服务器
 * 提供代码补全、悬停提示、诊断等功能
 */

import * as vscode from 'vscode';
import { LanguageServer } from './language-server-base';

export class BeejsLanguageServer extends LanguageServer {
    constructor() {
        super('beejs-language-server');
    }

    protected onInitialize() {
        console.log('Beejs 语言服务器已初始化');

        // 设置服务器能力
        this.client.onInitialize(() => {
            return {
                capabilities: {
                    // 文本同步
                    textDocument: {
                        synchronization: {
                            dynamicRegistration: true,
                            willSave: true,
                            didSave: true,
                            willSaveWaitUntil: true,
                        },
                    },
                    // 代码补全
                    completionProvider: {
                        resolveProvider: true,
                        triggerCharacters: ['.', '(', '[', '{', ',', ':'],
                    },
                    // 悬停
                    hoverProvider: true,
                    // 文档符号
                    documentSymbolProvider: true,
                    // 签名帮助
                    signatureHelpProvider: {
                        triggerCharacters: ['(', ',', '<'],
                    },
                    // 格式化
                    documentFormattingProvider: true,
                    documentRangeFormattingProvider: true,
                    // 重命名
                    renameProvider: {
                        prepareSupport: true,
                    },
                    // 代码操作
                    codeActionProvider: true,
                    // 引用查找
                    referencesProvider: true,
                },
            };
        });
    }

    protected onDidChangeConfiguration() {
        console.log('配置已更改');
    }

    protected onDidChangeWatchedFiles() {
        console.log('监听的文件已更改');
    }
}
