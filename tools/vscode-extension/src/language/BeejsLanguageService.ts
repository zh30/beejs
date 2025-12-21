/**
 * Beejs Language Service
 *
 * Provides language intelligence for JavaScript/TypeScript files
 * with Beejs-specific enhancements:
 * - Code completion
 * - Hover information
 * - Diagnostics
 * - Code actions
 */

import * as vscode from 'vscode';
import { LanguageClient, LanguageClientOptions, ServerOptions, TransportKind } from 'vscode-languageclient';
import { BeejsConfiguration } from '../utils/BeejsConfiguration';

export class BeejsLanguageService {
    private context: vscode.ExtensionContext;
    private config: BeejsConfiguration;
    private client: LanguageClient | undefined;

    constructor(context: vscode.ExtensionContext, config: BeejsConfiguration) {
        this.context = context;
        this.config = config;
    }

    public initialize(): LanguageClient {
        const serverModule = this.context.asAbsolutePath('./out/language/server.js');
        const debugOptions = { execArgv: ['--nolazy', '--inspect=6009'] };

        const serverOptions: ServerOptions = {
            run: {
                module: serverModule,
                transport: TransportKind.ipc,
            },
            debug: {
                module: serverModule,
                transport: TransportKind.ipc,
                options: debugOptions,
            },
        };

        const clientOptions: LanguageClientOptions = {
            documentSelector: [
                { scheme: 'file', language: 'javascript' },
                { scheme: 'file', language: 'typescript' },
                { scheme: 'file', language: 'beejs' },
            ],
            initializationOptions: {
                beejsPath: this.config.getRuntimePath(),
                enableTypeChecking: this.config.getEnableTypeChecking(),
                maxMemory: this.config.getMaxMemory(),
            },
            synchronize: {
                configurationSection: 'beejs',
            },
        };

        this.client = new LanguageClient(
            'beejs-language-server',
            'Beejs Language Server',
            serverOptions,
            clientOptions
        );

        // Start the client
        this.client.start();

        // Register additional providers
        this.registerProviders();

        return this.client;
    }

    private registerProviders() {
        if (!this.client) return;

        // Completion provider
        vscode.languages.registerCompletionItemProvider(
            ['javascript', 'typescript', 'beejs'],
            {
                provideCompletionItems: (document: vscode.TextDocument, position: vscode.Position) => {
                    // Provide Beejs-specific completions
                    const completions: vscode.CompletionItem[] = [];

                    // Beejs global API completions
                    completions.push({
                        label: 'beejs.run',
                        kind: vscode.CompletionItemKind.Function,
                        insertText: 'beejs.run(${1:script})',
                        detail: 'Execute a Beejs script',
                        documentation: 'Run a JavaScript/TypeScript script using Beejs runtime',
                        command: { command: 'editor.action.triggerSuggest', title: 'Re-trigger completions...' },
                    });

                    completions.push({
                        label: 'beejs.bundle',
                        kind: vscode.CompletionItemKind.Function,
                        insertText: 'beejs.bundle(${1:entry}, ${2:output})',
                        detail: 'Bundle scripts with Beejs',
                        documentation: 'Bundle JavaScript/TypeScript files into a single output',
                    });

                    completions.push({
                        label: 'beejs.test',
                        kind: vscode.CompletionItemKind.Function,
                        insertText: 'beejs.test(${1:pattern})',
                        detail: 'Run tests with Beejs',
                        documentation: 'Execute tests using Beejs test runner',
                    });

                    // Performance API
                    completions.push({
                        label: 'beejs.profile',
                        kind: vscode.CompletionItemKind.Function,
                        insertText: 'beejs.profile(() => {\n\t${1:// code}\n})',
                        detail: 'Profile code execution',
                        documentation: 'Profile code execution and get performance metrics',
                    });

                    completions.push({
                        label: 'beejs.benchmark',
                        kind: vscode.CompletionItemKind.Function,
                        insertText: 'beejs.benchmark(${1:fn}, ${2:iterations})',
                        detail: 'Benchmark function performance',
                        documentation: 'Benchmark a function with specified iterations',
                    });

                    // TypeScript-specific completions
                    if (document.languageId === 'typescript') {
                        completions.push({
                            label: 'beejs.compile',
                            kind: vscode.CompletionItemKind.Function,
                            insertText: 'beejs.compile(source, options)',
                            detail: 'Compile TypeScript with Beejs',
                            documentation: 'Compile TypeScript code using Beejs compiler',
                        });
                    }

                    return { completions };
                },
            },
            '.' // Trigger on dot
        );

        // Hover provider
        vscode.languages.registerHoverProvider(
            ['javascript', 'typescript', 'beejs'],
            {
                provideHover: (document: vscode.TextDocument, position: vscode.Position) => {
                    const word = document.getText(document.getWordRangeAtPosition(position));

                    if (word === 'beejs') {
                        return new vscode.Hover({
                            value: '**Beejs Runtime**\n\nHigh-performance JavaScript/TypeScript runtime',
                        });
                    }

                    if (word.startsWith('beejs.')) {
                        return new vscode.Hover({
                            value: `**${word}**\n\nBeejs API - [Learn more](https://beejs.dev/docs)`,
                        });
                    }

                    return undefined;
                },
            }
        );

        // Diagnostics provider
        vscode.languages.registerDiagnosticCollection(
            'beejs',
            vscode.window.activeTextEditor?.document.languageId === 'typescript'
        );

        // Code action provider
        vscode.languages.registerCodeActionsProvider(
            ['javascript', 'typescript', 'beejs'],
            {
                provideCodeActions: (document: vscode.TextDocument, range: vscode.Range) => {
                    const actions: vscode.CodeAction[] = [];

                    // Convert to Beejs script action
                    const convertAction = new vscode.CodeAction(
                        'Convert to Beejs format',
                        vscode.CodeActionKind.Refactor
                    );
                    convertAction.command = {
                        command: 'beejs.convertScript',
                        title: 'Convert to Beejs format',
                    };
                    actions.push(convertAction);

                    return actions;
                },
            }
        );
    }

    public dispose() {
        if (this.client) {
            this.client.stop();
        }
    }
}
