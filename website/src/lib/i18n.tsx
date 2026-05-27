import React, { createContext, useContext, useEffect, useMemo, useState } from 'react'

export type Lang = 'en' | 'zh'

type LangContextValue = {
  lang: Lang
  setLang: (lang: Lang) => void
  toggle: () => void
  copy: (typeof copy)[Lang]
}

const copy = {
  en: {
    nav: {
      home: 'Home',
      docs: 'Manual',
      blog: 'Release Notes',
      github: 'GitHub',
    },
    toggle: {
      label: 'Language',
      en: 'EN',
      zh: '中文',
    },
    footer: {
      statusLabel: 'System Status',
      statusValue: 'Operational',
      stage: 'v0.1.0',
      contact: 'Contact',
      email: 'support@beejs.zhanghe.dev',
      rights: 'All rights reserved.',
    },
    home: {
      kicker: 'Runtime Console',
      title: 'Beejs Runtime',
      titleAccent: 'Core',
      subtitle:
        'JavaScript/TypeScript runtime built with Rust + V8. Public v0.1 focuses on a clean CLI, script execution, TypeScript transpilation, and core compatibility.',
      ctaPrimary: 'Open Manual',
      ctaSecondary: 'Read Notes',
      heroMetricLabel: 'Current Release',
      heroMetricValue: 'v0.1',
      heroMetricUnit: 'core',
      heroMetricNote: 'Cargo package version 0.1.0',
      heroFootnote: 'Public Runtime Scope',
      telemetryTitle: 'Release Scope',
      telemetrySubtitle: 'Built around verified defaults.',
      telemetryNote: 'These are product-surface checks, not synthetic benchmark claims.',
      telemetry: [
        { label: 'Runtime', value: 'Rust+V8', delta: 'core', note: 'execution' },
        { label: 'TypeScript', value: 'TS/TSX', delta: 'built-in', note: 'transpile' },
        { label: 'Platforms', value: 'macOS/Linux', delta: 'prebuilt', note: 'assets' },
        { label: 'Output', value: 'quiet', delta: 'default', note: 'CLI' },
      ],
      benchmarksTitle: 'Verification',
      benchmarksSubtitle: 'Release gates match the default build.',
      benchmarksMeta: 'CI scope',
      benchmarks: [
        { label: 'Format', value: 'fmt', unit: 'checked', delta: 'cargo fmt' },
        { label: 'Lint', value: 'clippy', unit: 'clean', delta: '-D warnings' },
        { label: 'Core Tests', value: 'lib+CLI', unit: 'covered', delta: 'release suite' },
      ],
      systemsTitle: 'Core Systems',
      systemsSubtitle: 'Current public runtime surface.',
      systemsMeta: 'Subsystem map',
      systemsLabel: 'module',
      systems: [
        {
          title: 'V8 Runtime',
          desc: 'Executes JavaScript through the active minimal runtime used by the CLI.',
        },
        {
          title: 'TypeScript Loader',
          desc: 'Transpiles TS and TSX files before execution.',
        },
        {
          title: 'Node Compatibility',
          desc: 'Provides selected fs, path, crypto, buffer, process, timers, and CommonJS APIs.',
        },
        {
          title: 'Web API Layer',
          desc: 'Includes selected fetch, URL, streams, crypto, blob, event, and worker APIs.',
        },
        {
          title: 'Test Runner',
          desc: 'Runs Jest-style test files and focused runtime checks.',
        },
        {
          title: 'Package Tools',
          desc: 'Supports init, add, install, prune, and related package commands.',
        },
      ],
      ctaTitle: 'Install Beejs',
      ctaSubtitle: 'Open-source v0.1 runtime for macOS and Linux.',
      ctaButton: 'Read Install Guide',
    },
    docs: {
      title: 'Runtime Manual',
      subtitle: 'Operator documentation for Beejs v0.1.',
      backToHome: 'Return Home',
      groups: [
        {
          title: 'Start',
          items: [
            { id: 'introduction', label: 'Overview' },
            { id: 'installation', label: 'Installation' },
            { id: 'quick-start', label: 'Quick Start' },
          ],
        },
        {
          title: 'Runtime',
          items: [
            { id: 'v8-isolate-pool', label: 'Runtime Core' },
            { id: 'jit-optimization', label: 'TypeScript' },
            { id: 'memory-management', label: 'Compatibility' },
            { id: 'server-mode', label: 'Serve Mode' },
          ],
        },
        {
          title: 'Operations',
          items: [
            { id: 'cli-usage', label: 'CLI Usage' },
            { id: 'api-reference', label: 'API Surface' },
            { id: 'modules', label: 'Modules' },
          ],
        },
      ],
      sections: {
        introduction: {
          title: 'Overview',
          subtitle: 'Rust + V8 runtime for JavaScript and TypeScript.',
          body: [
            'Beejs v0.1 is the public core release of the runtime. It keeps the default surface focused on script execution, eval, REPL usage, TypeScript transpilation, and core compatibility APIs.',
            'The repository also contains historical stage reports and feature-gated modules. Those documents are useful for design history, but the public release promise follows the default Cargo build.',
          ],
          cards: [
            { title: 'Clean CLI', desc: 'Default run and eval output avoids internal setup logs.' },
            { title: 'Default Build', desc: 'Release checks target the same feature set users install.' },
          ],
        },
        installation: {
          title: 'Installation',
          subtitle: 'Install a prebuilt archive or build from source.',
          body: [
            'Prebuilt release archives currently target macOS x86_64, macOS arm64, and Linux x86_64. Other platforms can build from source with Rust.',
          ],
          code: [
            '$ curl -fsSL https://beejs.zhanghe.dev/install.sh | sh',
            '$ bee --version',
          ],
        },
        'quick-start': {
          title: 'Quick Start',
          subtitle: 'Run your first script.',
          body: ['Create a JavaScript or TypeScript file and execute it with the run subcommand.'],
          code: [
            'console.log("Hello from Beejs");',
            'bee run hello.js',
            'bee eval "1 + 1"',
          ],
        },
        'v8-isolate-pool': {
          title: 'Runtime Core',
          subtitle: 'The active CLI path uses V8 through Rust.',
          body: [
            'The default binary entry is src/main.rs. Script execution is handled by src/runtime_minimal.rs, which owns the V8 isolate, context setup, and result handling.',
          ],
          list: [
            'Execute JavaScript files with bee run',
            'Evaluate snippets with bee eval',
            'Use bee repl for an interactive shell',
          ],
        },
        'jit-optimization': {
          title: 'TypeScript',
          subtitle: 'TS and TSX files are transpiled before execution.',
          body: [
            'When a .ts or .tsx file is passed to bee run, the CLI reads the source and routes it through the TypeScript module before V8 execution.',
          ],
          list: [
            'Use .ts and .tsx entry files',
            'Keep runtime-specific behavior covered by executable tests',
          ],
        },
        'memory-management': {
          title: 'Compatibility',
          subtitle: 'Selected Node.js and Web APIs are available.',
          body: [
            'The default build includes compatibility layers for common Node.js and Web APIs. Coverage is partial and should be checked against examples or tests before relying on a specific edge case.',
          ],
          list: [
            'Node.js modules include fs, path, crypto, buffer, process, timers, and require',
            'Web APIs include fetch, URL, streams, Blob, events, timers, and Web Crypto pieces',
          ],
        },
        'server-mode': {
          title: 'Serve Mode',
          subtitle: 'Start the built-in HTTP or HTTPS server.',
          body: [
            'The CLI exposes serve for lightweight runtime server usage. HTTPS requires explicit certificate and key paths.',
          ],
          code: ['$ bee serve --host localhost --port 3000'],
        },
        'cli-usage': {
          title: 'CLI Usage',
          subtitle: 'Core commands.',
          list: [
            'bee run <file> - execute a JavaScript or TypeScript file',
            'bee eval <code> - evaluate a JavaScript snippet',
            'bee test [file] - run the built-in or file-based test runner',
            'bee bundle <entry> - write a production bundle',
            'bee serve - start the HTTP or HTTPS server',
            'bee install - install dependencies from package.json',
          ],
        },
        'api-reference': {
          title: 'API Surface',
          subtitle: 'Check behavior against the current runtime.',
          body: [
            'Beejs exposes a practical subset of Node.js and Web platform APIs. The safest reference is the executable test suite and the examples directory.',
          ],
          list: ['console and timers', 'CommonJS require', 'fetch and URL', 'fs, path, crypto, buffer, process'],
        },
        modules: {
          title: 'Modules',
          subtitle: 'Default module boundaries.',
          list: [
            'src/runtime_minimal.rs - current V8 runtime',
            'src/nodejs_core/ - Node.js compatibility modules',
            'src/web_api/ - Web API modules',
            'src/testing/ - test framework',
            'src/package_manager.rs - package manager support',
          ],
        },
      },
    },
    blog: {
      title: 'Release Notes',
      subtitle: 'Runtime notes, implementation updates, and release scope.',
      tagLabel: 'Topic',
      back: 'Return to Notes',
      operator: 'Author',
      timestamp: 'Date',
      readTime: 'Read Time',
      readMore: 'Open Note',
      notFound: 'Post Not Found',
    },
  },
  zh: {
    nav: {
      home: '首页',
      docs: '手册',
      blog: '发布记录',
      github: 'GitHub',
    },
    toggle: {
      label: '语言',
      en: 'EN',
      zh: '中文',
    },
    footer: {
      statusLabel: '系统状态',
      statusValue: '运行中',
      stage: 'v0.1.0',
      contact: '联系',
      email: 'support@beejs.zhanghe.dev',
      rights: '保留所有权利。',
    },
    home: {
      kicker: '运行时控制台',
      title: 'Beejs 运行时',
      titleAccent: '核心版',
      subtitle: 'Rust + V8 构建的 JavaScript/TypeScript 运行时。公开 v0.1 聚焦干净 CLI、脚本执行、TypeScript 转译和核心兼容能力。',
      ctaPrimary: '打开手册',
      ctaSecondary: '查看记录',
      heroMetricLabel: '当前版本',
      heroMetricValue: 'v0.1',
      heroMetricUnit: 'core',
      heroMetricNote: 'Cargo package version 0.1.0',
      heroFootnote: '公开运行时范围',
      telemetryTitle: '发布范围',
      telemetrySubtitle: '围绕默认构建做验证。',
      telemetryNote: '这里展示的是产品表面和验证闸门，不是合成性能宣传。',
      telemetry: [
        { label: '运行时', value: 'Rust+V8', delta: '核心', note: '执行' },
        { label: 'TypeScript', value: 'TS/TSX', delta: '内置', note: '转译' },
        { label: '平台', value: 'macOS/Linux', delta: '预编译', note: '产物' },
        { label: '输出', value: 'quiet', delta: '默认', note: 'CLI' },
      ],
      benchmarksTitle: '验证',
      benchmarksSubtitle: '发布闸门匹配默认构建。',
      benchmarksMeta: 'CI 范围',
      benchmarks: [
        { label: '格式', value: 'fmt', unit: 'checked', delta: 'cargo fmt' },
        { label: 'Lint', value: 'clippy', unit: 'clean', delta: '-D warnings' },
        { label: '核心测试', value: 'lib+CLI', unit: 'covered', delta: 'release suite' },
      ],
      systemsTitle: '核心系统',
      systemsSubtitle: '当前公开运行时表面。',
      systemsMeta: '子系统地图',
      systemsLabel: '模块',
      systems: [
        {
          title: 'V8 运行时',
          desc: '通过当前 CLI 使用的 minimal runtime 执行 JavaScript。',
        },
        {
          title: 'TypeScript 加载',
          desc: '执行前转译 TS 和 TSX 文件。',
        },
        {
          title: 'Node 兼容层',
          desc: '提供 fs、path、crypto、buffer、process、timers 和 CommonJS 等选定 API。',
        },
        {
          title: 'Web API 层',
          desc: '包含 fetch、URL、streams、crypto、Blob、events 和 worker 等选定 API。',
        },
        {
          title: '测试运行器',
          desc: '运行 Jest 风格测试文件和聚焦运行时检查。',
        },
        {
          title: '包管理工具',
          desc: '支持 init、add、install、prune 等包管理命令。',
        },
      ],
      ctaTitle: '安装 Beejs',
      ctaSubtitle: '面向 macOS 和 Linux 的开源 v0.1 运行时。',
      ctaButton: '查看安装指南',
    },
    docs: {
      title: '运行时手册',
      subtitle: 'Beejs v0.1 操作文档。',
      backToHome: '返回首页',
      groups: [
        {
          title: '开始',
          items: [
            { id: 'introduction', label: '概览' },
            { id: 'installation', label: '安装' },
            { id: 'quick-start', label: '快速开始' },
          ],
        },
        {
          title: '运行时',
          items: [
            { id: 'v8-isolate-pool', label: '运行时核心' },
            { id: 'jit-optimization', label: 'TypeScript' },
            { id: 'memory-management', label: '兼容层' },
            { id: 'server-mode', label: 'Serve 模式' },
          ],
        },
        {
          title: '运行维护',
          items: [
            { id: 'cli-usage', label: 'CLI 用法' },
            { id: 'api-reference', label: 'API 表面' },
            { id: 'modules', label: '模块' },
          ],
        },
      ],
      sections: {
        introduction: {
          title: '概览',
          subtitle: 'Rust + V8 构建的 JavaScript 和 TypeScript 运行时。',
          body: [
            'Beejs v0.1 是运行时的公开核心版本。默认能力聚焦脚本执行、eval、REPL、TypeScript 转译和核心兼容 API。',
            '仓库仍保留历史阶段报告和 feature-gated 模块。这些资料适合了解设计背景，但公开发布承诺以默认 Cargo 构建为准。',
          ],
          cards: [
            { title: '干净 CLI', desc: '默认 run 和 eval 输出不泄漏内部初始化日志。' },
            { title: '默认构建', desc: '发布检查覆盖用户实际安装的 feature 集。' },
          ],
        },
        installation: {
          title: '安装',
          subtitle: '使用预编译包或从源码构建。',
          body: ['预编译发布产物当前覆盖 macOS x86_64、macOS arm64 和 Linux x86_64。其他平台可通过 Rust 从源码构建。'],
          code: [
            '$ curl -fsSL https://beejs.zhanghe.dev/install.sh | sh',
            '$ bee --version',
          ],
        },
        'quick-start': {
          title: '快速开始',
          subtitle: '运行第一段脚本。',
          body: ['创建 JavaScript 或 TypeScript 文件，并通过 run 子命令执行。'],
          code: ['console.log("Hello from Beejs");', 'bee run hello.js', 'bee eval "1 + 1"'],
        },
        'v8-isolate-pool': {
          title: '运行时核心',
          subtitle: '当前 CLI 路径通过 Rust 驱动 V8。',
          body: ['默认二进制入口是 src/main.rs。脚本执行由 src/runtime_minimal.rs 处理，负责 V8 isolate、上下文初始化和结果返回。'],
          list: ['用 bee run 执行 JavaScript 文件', '用 bee eval 执行片段', '用 bee repl 进入交互式终端'],
        },
        'jit-optimization': {
          title: 'TypeScript',
          subtitle: 'TS 和 TSX 文件执行前会先转译。',
          body: ['当 .ts 或 .tsx 文件传给 bee run 时，CLI 会读取源码并通过 TypeScript 模块处理，再交给 V8 执行。'],
          list: ['可使用 .ts 和 .tsx 入口文件', '运行时行为应通过可执行测试覆盖'],
        },
        'memory-management': {
          title: '兼容层',
          subtitle: '提供选定 Node.js 和 Web API。',
          body: ['默认构建包含常见 Node.js 与 Web API 兼容层。覆盖并非完整标准实现，依赖具体边界前应查看示例或测试。'],
          list: ['Node.js 模块包括 fs、path、crypto、buffer、process、timers 和 require', 'Web API 包括 fetch、URL、streams、Blob、events、timers 和部分 Web Crypto'],
        },
        'server-mode': {
          title: 'Serve 模式',
          subtitle: '启动内置 HTTP 或 HTTPS 服务。',
          body: ['CLI 提供 serve 用于轻量服务场景。HTTPS 需要显式传入证书和私钥路径。'],
          code: ['$ bee serve --host localhost --port 3000'],
        },
        'cli-usage': {
          title: 'CLI 用法',
          subtitle: '核心命令。',
          list: [
            'bee run <file> - 执行 JavaScript 或 TypeScript 文件',
            'bee eval <code> - 执行 JavaScript 片段',
            'bee test [file] - 运行内置或文件测试',
            'bee bundle <entry> - 写出生产 bundle',
            'bee serve - 启动 HTTP 或 HTTPS 服务',
            'bee install - 从 package.json 安装依赖',
          ],
        },
        'api-reference': {
          title: 'API 表面',
          subtitle: '以当前运行时行为为准。',
          body: ['Beejs 暴露实用的 Node.js 和 Web 平台 API 子集。最稳妥的参考是可执行测试和 examples 目录。'],
          list: ['console 与 timers', 'CommonJS require', 'fetch 与 URL', 'fs、path、crypto、buffer、process'],
        },
        modules: {
          title: '模块',
          subtitle: '默认模块边界。',
          list: [
            'src/runtime_minimal.rs - 当前 V8 运行时',
            'src/nodejs_core/ - Node.js 兼容模块',
            'src/web_api/ - Web API 模块',
            'src/testing/ - 测试框架',
            'src/package_manager.rs - 包管理支持',
          ],
        },
      },
    },
    blog: {
      title: '发布记录',
      subtitle: '运行时说明、实现更新和发布范围。',
      tagLabel: '主题',
      back: '返回记录',
      operator: '作者',
      timestamp: '日期',
      readTime: '阅读耗时',
      readMore: '打开记录',
      notFound: '未找到文章',
    },
  },
} as const

const LangContext = createContext<LangContextValue | null>(null)

export function LangProvider({ children }: { children: React.ReactNode }) {
  const [lang, setLang] = useState<Lang>(() => {
    if (typeof window === 'undefined') return 'en'
    const stored = window.localStorage.getItem('beejs_lang')
    if (stored === 'en' || stored === 'zh') return stored
    const browser = window.navigator?.language?.toLowerCase() || 'en'
    return browser.startsWith('zh') ? 'zh' : 'en'
  })

  useEffect(() => {
    if (typeof window !== 'undefined') {
      window.localStorage.setItem('beejs_lang', lang)
    }
  }, [lang])

  const value = useMemo<LangContextValue>(() => {
    return {
      lang,
      setLang,
      toggle: () => setLang(lang === 'en' ? 'zh' : 'en'),
      copy: copy[lang],
    }
  }, [lang])

  return <LangContext.Provider value={value}>{children}</LangContext.Provider>
}

export function useLang() {
  const ctx = useContext(LangContext)
  if (!ctx) {
    throw new Error('useLang must be used within LangProvider')
  }
  return ctx
}
