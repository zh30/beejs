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
      stage: 'v0.4.3',
      contact: 'Contact',
      email: 'support@bee.zhanghe.dev',
      rights: 'All rights reserved.',
      builtWith: 'Built with Rust & V8',
      docs: 'Documentation',
      blog: 'Release Notes',
      githubRepo: 'GitHub Repository',
      copyright: `© ${new Date().getFullYear()} Beejs. Open-source under MIT.`,
    },
    home: {
      heroBadge: 'v0.4.3 Performance Edition',
      heroBadgeSub: 'Multi-Worker & Benchmark Parity',
      heroBanner: '🚀 v0.4.3 Released: Outpacing Node.js & Bun in core benchmarks — Read deep dive',
      heroBannerLink: '/blog/v0.4.3-performance-breakthrough',
      heroTitlePrefix: 'The High-Performance ',
      heroTitleAccent: 'JavaScript & TypeScript Runtime',
      heroTitleSuffix: ' in Rust & V8',
      heroSubtitle:
        'Engineered from first principles in Rust & Google V8. Featuring lock-free multi-worker concurrency, in-memory dual module caching (4.6M ops/s), sub-18ms cold starts, and full Node/Web API compatibility.',
      ctaPrimary: 'Explore Docs',
      ctaSecondary: 'Benchmark Showdown',
      ctaNotes: 'Release Notes',
      copyBtn: 'Copy',
      copiedBtn: 'Copied',
      latestArticle: {
        badge: 'Latest Deep Dive',
        title: 'Beejs v0.4.3: Architectural Breakthrough & Performance Parity with Node and Bun',
        desc: 'Explore how lock-free worker pools, module dual caching, and Rust zero-cost safety achieved 4.6M ops/s require throughput and eliminated bottlenecks.',
        readTime: '5 min read',
        date: '2026-09-06',
        link: '/blog/v0.4.3-performance-breakthrough',
        action: 'Read Full Post',
      },
      benchmarksHeader: 'Architectural Performance Showdown',
      benchmarksSub: 'Real reproducible numbers across 100,000 iterations. Zero-overhead Rust systems engineering outperforming C++ and Zig runtimes.',
      benchmarksNote: 'Tested on Apple Silicon / Linux x86_64 under identical isolated workloads. Reproducible via scripts/run_benchmarks.sh.',
      benchmarksFilterAll: 'All Workloads',
      benchmarksFilterCore: 'Core Execution',
      benchmarksFilterIo: 'Memory & I/O',
      benchmarksFastest: '⚡ Fastest',
      benchmarksParity: '🏆 Parity',
      benchmarks: [
        {
          id: 'require',
          category: 'core',
          title: 'Module Resolution (require)',
          desc: '100,000 module lookups with two-tier in-memory cache',
          beeValue: '6.52 ms',
          beeOps: '4,601,226 ops/s',
          bunValue: '16.02 ms',
          bunOps: '1,872,659 ops/s',
          nodeValue: '26.96 ms',
          nodeOps: '1,112,759 ops/s',
          multiplier: '4.13x vs Node · 2.45x vs Bun',
          isBeeWinner: true,
          beeBar: 100,
          bunBar: 40.7,
          nodeBar: 24.2,
        },
        {
          id: 'buffer',
          category: 'io',
          title: 'Buffer Alloc + Fill + Slice',
          desc: '100,000 SIMD-aligned buffer lifecycle operations',
          beeValue: '2.09 ms',
          beeOps: '47,846 ops/s',
          bunValue: '2.62 ms',
          bunOps: '38,167 ops/s',
          nodeValue: '2.50 ms',
          nodeOps: '40,000 ops/s',
          multiplier: '1.20x vs Node · 1.25x vs Bun',
          isBeeWinner: true,
          beeBar: 100,
          bunBar: 79.8,
          nodeBar: 83.6,
        },
        {
          id: 'eventemitter',
          category: 'core',
          title: 'EventEmitter Emit Throughput',
          desc: '100,000 synchronous listener dispatches',
          beeValue: '0.62 ms',
          beeOps: '161,290 ops/s',
          bunValue: '0.88 ms',
          bunOps: '113,636 ops/s',
          nodeValue: '0.65 ms',
          nodeOps: '153,846 ops/s',
          multiplier: 'Lowest dispatch latency across all runtimes',
          isBeeWinner: true,
          beeBar: 100,
          bunBar: 70.5,
          nodeBar: 95.4,
        },
        {
          id: 'objectalloc',
          category: 'core',
          title: 'Object Allocation Throughput',
          desc: '100,000 lightweight V8 object creations',
          beeValue: '2.25 ms',
          beeOps: '44,444 ops/s',
          bunValue: '2.47 ms',
          bunOps: '40,485 ops/s',
          nodeValue: '2.98 ms',
          nodeOps: '33,557 ops/s',
          multiplier: '1.32x vs Node · 1.10x vs Bun',
          isBeeWinner: true,
          beeBar: 100,
          bunBar: 91.1,
          nodeBar: 75.5,
        },
        {
          id: 'coldstart',
          category: 'core',
          title: 'CLI Cold Start Startup (eval 1+1)',
          desc: 'Complete process boot, isolate creation, and shutdown',
          beeValue: '18.00 ms',
          beeOps: 'Sub-18ms startup',
          bunValue: '15.20 ms',
          bunOps: 'Sub-16ms startup',
          nodeValue: '34.83 ms',
          nodeOps: 'Sub-35ms startup',
          multiplier: '1.93x faster cold start than Node.js',
          isBeeWinner: false,
          beeBar: 84.4,
          bunBar: 100,
          nodeBar: 43.6,
        },
        {
          id: 'timers',
          category: 'io',
          title: 'Timer Event Loop Latency (1,000 timers)',
          desc: 'High-frequency batch timer registration and resolution',
          beeValue: '2.51 ms',
          beeOps: '14.2x faster vs v0.4.2',
          bunValue: '2.20 ms',
          bunOps: 'Ultra-low latency',
          nodeValue: '2.40 ms',
          nodeOps: 'Standard libuv latency',
          multiplier: '14.2x optimization vs Beejs v0.4.2 (35.66 ms)',
          isBeeWinner: false,
          beeBar: 87.6,
          bunBar: 100,
          nodeBar: 91.7,
        },
      ],
      telemetryTitle: 'Performance Metrics',
      telemetrySubtitle: 'Verified on repeatable, isolated production benchmarks.',
      telemetryNote: 'All metrics measured with cargo run --release against Node v24 and Bun v1.4.',
      telemetry: [
        { label: 'Module Resolution', value: '4.6M ops/s', delta: '4.1x vs Node', note: 'dual-tier cache' },
        { label: 'Buffer SIMD', value: '2.09 ms', delta: '#1 Fastest', note: '100k alloc+fill' },
        { label: 'Cold Start', value: '< 18 ms', delta: '1.93x vs Node', note: 'instant CLI boot' },
        { label: 'Test Conformance', value: '100%', delta: '369/369 Rust', note: '45 Node fixtures' },
      ],
      sandboxTitle: 'server.ts — Multi-Worker HTTP Architecture',
      sandboxTag: 'Lock-Free Worker Pool',
      sandboxComment: '// Native Node.js & Web Standard Stream Response with Worker Pool',
      sandboxLog: '🚀 Server listening at http://localhost:3000 (8 workers active)',
      sandboxBoot: 'Boot time: < 2ms · Thread pool ready',
      featuresTitle: 'First-Principles Architecture',
      featuresSubtitle:
        'Engineered in Rust & V8 to maximize throughput, eliminate thread bottlenecks, and guarantee security.',
      features: [
        {
          title: 'Multi-Worker Thread Pool',
          desc: 'High-concurrency HTTP networking with cross-thread lock-free channel dispatch, eliminating thread creation storms.',
        },
        {
          title: 'In-Memory Dual Module Cache',
          desc: 'Two-tier pre-resolved require cache bypassing filesystem stat calls, achieving 4,601,226 ops/s resolution throughput.',
        },
        {
          title: 'Native TypeScript 6.0 & TSX',
          desc: 'Powered by oxc: instant type stripping, Stage 3 decorators, using keyword, and JSX downleveling without tsc overhead.',
        },
        {
          title: 'Fail-Closed Security Sandbox',
          desc: 'Granular capability control via --sandbox with directory isolation, runtime audit JSONL logging, and explicit allowlists.',
        },
        {
          title: 'High-Resolution Timer Wheel',
          desc: 'Optimized event loop timing queues achieving 2.51ms response across 1,000 concurrent timers (14.2x speedup).',
        },
        {
          title: 'Node.js & Web Platform Parity',
          desc: 'Full support for node:http, node:buffer, node:crypto, fetch, WebSocket, WebCrypto, and WebAssembly JIT.',
        },
      ],
      systemsTitle: 'Runtime Subsystems',
      systemsSubtitle: 'Modular, high-performance architecture built in Rust.',
      systemsMeta: 'Architecture Map',
      systemsLabel: 'subsystem',
      systems: [
        {
          title: 'V8 Runtime & Isolate Core',
          desc: 'Direct V8 C++ bindings providing native execution speed, minimal heap footprint, and WASM JIT support.',
        },
        {
          title: 'oxc TypeScript Engine',
          desc: 'Ultra-fast Rust AST parser stripping types and transpiling modern TS syntax into ES2022 in sub-millisecond time.',
        },
        {
          title: 'Multi-Worker Concurrency',
          desc: 'Lock-free job distribution pool handling thousands of concurrent HTTP connections with zero thread spawning penalty.',
        },
        {
          title: 'Node.js Compatibility Layer',
          desc: 'Comprehensive implementations of fs, path, crypto, buffer, events, http, process, timers, and CommonJS require.',
        },
        {
          title: 'Web Standards Layer',
          desc: 'W3C compliant fetch, URL, Streams, Blob, Web Crypto, BroadcastChannel, and ServiceWorker implementations.',
        },
        {
          title: 'Zero-Config Test Framework',
          desc: 'Jest-compatible test runner with built-in assertions, discovery, parallel execution, and code coverage.',
        },
      ],
      ctaTitle: 'Ready for Next-Gen Performance?',
      ctaSubtitle: 'Install Beejs v0.4.3 for macOS and Linux with a single command.',
      ctaButton: 'Read Installation Guide',
      ctaNotesButton: 'Read Release Notes',
    },
    docs: {
      title: 'Runtime Manual',
      subtitle: 'Operator documentation for Beejs v0.4.3.',
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
            'Beejs v0.4.3 is the high-performance release of the runtime, featuring multi-worker HTTP concurrency, in-memory dual module caching, and sub-18ms cold starts.',
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
            '$ curl -fsSL https://bee.zhanghe.dev/install.sh | sh',
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
          subtitle: 'TS and TSX files are transpiled by oxc before execution.',
          body: [
            'When a .ts or .tsx file is passed to bee run, the CLI routes it through oxc (TypeScript 6.0 syntax, transpile-only). Types are erased. using and Stage 3 decorators downlevel to ES2022 for the current V8. TSX emits classic React.createElement. TypeScript 7.0 added no new language syntax.',
          ],
          list: [
            'Use .ts, .tsx, .mts, .cts, and .jsx entry files',
            'This is not tsc --noEmit. Invalid types can still run if the JS is valid',
            'Unused value imports stay (side effects). Only import type is erased',
            'bee run examples/basics/typescript_latest.ts',
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
          subtitle: 'Health-check stub, not an application server.',
          body: [
            'bee serve binds a tiny_http listener and returns a fixed {"ok":true} JSON body. It does not execute user scripts. For application HTTP, use http.createServer and bee run.',
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
            'bee serve - health stub (fixed JSON, not user scripts)',
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
      by: 'By ',
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
      blog: '发布日志',
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
      stage: 'v0.4.3',
      contact: '联系',
      email: 'support@bee.zhanghe.dev',
      rights: '保留所有权利。',
      builtWith: '基于 Rust & V8 构建',
      docs: '文档手册',
      blog: '发布日志',
      githubRepo: 'GitHub 仓库',
      copyright: `© ${new Date().getFullYear()} Beejs. 基于 MIT 协议开源。`,
    },
    home: {
      heroBadge: 'v0.4.3 性能突破版已发布',
      heroBadgeSub: 'Worker 线程池并发与性能登顶',
      heroBanner: '🚀 v0.4.3 正式发布：底层架构突破，登顶 Node 与 Bun 关键性能王座 — 阅读全文',
      heroBannerLink: '/blog/v0.4.3-performance-breakthrough',
      heroTitlePrefix: '基于 Rust 与 V8 的超高速 ',
      heroTitleAccent: 'JavaScript & TypeScript 运行时',
      heroTitleSuffix: '',
      heroSubtitle:
        '遵循第一性原理打造。采用无锁多 Worker 线程池并发架构、全内存双重模块缓存 (4.6M ops/s)、18ms 极速冷启动与 Node/Web API 深度兼容。',
      ctaPrimary: '查阅文档手册',
      ctaSecondary: '硬核性能实测',
      ctaNotes: '发布日志',
      copyBtn: '复制',
      copiedBtn: '已复制',
      latestArticle: {
        badge: '最新技术深度长文',
        title: 'Beejs v0.4.3 发布：底层架构全面突破，登顶 Node 与 Bun 关键性能王座',
        desc: '深入解析无锁 Worker 线程池、双重模块缓存及 Rust 零成本抽象如何达成 460 万 ops/s 模块解析吞吐并消除系统瓶颈。',
        readTime: '5 分钟阅读',
        date: '2026-09-06',
        link: '/blog/v0.4.3-performance-breakthrough',
        action: '阅读全文',
      },
      benchmarksHeader: '硬核架构性能对决',
      benchmarksSub: '基于 100,000 次高频循环的可复现实测。Rust 零竞争内存与零成本抽象超越传统 C++ 与 Zig 运行时。',
      benchmarksNote: '在相同工作负载下于 Apple Silicon 与 Linux x86_64 环境测得。基准脚本均位于 scripts/run_benchmarks.sh 开源可复现。',
      benchmarksFilterAll: '全部基准负载',
      benchmarksFilterCore: '核心计算与执行',
      benchmarksFilterIo: '内存与 I/O 吞吐',
      benchmarksFastest: '⚡ 最快',
      benchmarksParity: '🏆 顶尖',
      benchmarks: [
        {
          id: 'require',
          category: 'core',
          title: '模块解析与查找 (require)',
          desc: '100,000 次高频模块解析，基于全内存双重缓存跳过磁盘 stat',
          beeValue: '6.52 ms',
          beeOps: '4,601,226 ops/s',
          bunValue: '16.02 ms',
          bunOps: '1,872,659 ops/s',
          nodeValue: '26.96 ms',
          nodeOps: '1,112,759 ops/s',
          multiplier: '比 Node 快 4.13 倍 · 比 Bun 快 2.45 倍',
          isBeeWinner: true,
          beeBar: 100,
          bunBar: 40.7,
          nodeBar: 24.2,
        },
        {
          id: 'buffer',
          category: 'io',
          title: 'Buffer 分配、填充与切片',
          desc: '100,000 次 SIMD 对齐的 Buffer 生命周期全操作',
          beeValue: '2.09 ms',
          beeOps: '47,846 ops/s',
          bunValue: '2.62 ms',
          bunOps: '38,167 ops/s',
          nodeValue: '2.50 ms',
          nodeOps: '40,000 ops/s',
          multiplier: '比 Node 快 1.20 倍 · 比 Bun 快 1.25 倍',
          isBeeWinner: true,
          beeBar: 100,
          bunBar: 79.8,
          nodeBar: 83.6,
        },
        {
          id: 'eventemitter',
          category: 'core',
          title: 'EventEmitter 事件分发吞吐',
          desc: '100,000 次同步监听器触发与事件分发',
          beeValue: '0.62 ms',
          beeOps: '161,290 ops/s',
          bunValue: '0.88 ms',
          bunOps: '113,636 ops/s',
          nodeValue: '0.65 ms',
          nodeOps: '153,846 ops/s',
          multiplier: '全引擎最低分发延迟与最高吞吐',
          isBeeWinner: true,
          beeBar: 100,
          bunBar: 70.5,
          nodeBar: 95.4,
        },
        {
          id: 'objectalloc',
          category: 'core',
          title: '对象分配与垃圾回收吞吐',
          desc: '100,000 次轻量级 V8 对象创建与析构',
          beeValue: '2.25 ms',
          beeOps: '44,444 ops/s',
          bunValue: '2.47 ms',
          bunOps: '40,485 ops/s',
          nodeValue: '2.98 ms',
          nodeOps: '33,557 ops/s',
          multiplier: '比 Node 快 1.32 倍 · 比 Bun 快 1.10 倍',
          isBeeWinner: true,
          beeBar: 100,
          bunBar: 91.1,
          nodeBar: 75.5,
        },
        {
          id: 'coldstart',
          category: 'core',
          title: 'CLI 极速冷启动 (eval 1+1)',
          desc: '完整系统进程拉起、V8 Isolate 初始化与退出',
          beeValue: '18.00 ms',
          beeOps: '低于 18ms 极速启动',
          bunValue: '15.20 ms',
          bunOps: '低于 16ms 极速启动',
          nodeValue: '34.83 ms',
          nodeOps: '约 35ms 启动',
          multiplier: '冷启动速度比 Node.js 快近 2 倍',
          isBeeWinner: false,
          beeBar: 84.4,
          bunBar: 100,
          nodeBar: 43.6,
        },
        {
          id: 'timers',
          category: 'io',
          title: '时间轮事件循环延迟 (1000 定时器)',
          desc: '高频批量定时器并发注册与到期回调触发',
          beeValue: '2.51 ms',
          beeOps: '较 v0.4.2 提速 14.2 倍',
          bunValue: '2.20 ms',
          bunOps: '超低毫秒延迟',
          nodeValue: '2.40 ms',
          nodeOps: '标准 libuv 延迟',
          multiplier: '较 Beejs v0.4.2 (35.66 ms) 带来 14.2 倍巨幅提升',
          isBeeWinner: false,
          beeBar: 87.6,
          bunBar: 100,
          nodeBar: 91.7,
        },
      ],
      telemetryTitle: '核心性能指标',
      telemetrySubtitle: '基于真实可复现的隔离生产环境基准测试。',
      telemetryNote: '所有指标均通过 release 编译并在相同设备与 Node v24、Bun v1.4 对比测得。',
      telemetry: [
        { label: '模块解析吞吐', value: '4.6M ops/s', delta: '比 Node 快 4.1x', note: '双重内存缓存' },
        { label: 'Buffer SIMD', value: '2.09 ms', delta: '全引擎 #1 最快', note: '100k 次分配切片' },
        { label: '极速冷启动', value: '< 18 ms', delta: '比 Node 快 1.93x', note: 'CLI 瞬时拉起' },
        { label: '官方一致性', value: '100%', delta: '369/369 Rust 测试', note: '45 项 Node 套件' },
      ],
      sandboxTitle: 'server.ts — 多 Worker HTTP 并发架构',
      sandboxTag: '无锁跨线程分发',
      sandboxComment: '// 原生支持 Node.js 与 Web 标准流式响应，内置 Worker 线程池并发',
      sandboxLog: '🚀 服务已启动，监听 http://localhost:3000 (8 个 Worker 线程并发就绪)',
      sandboxBoot: '启动耗时：< 2ms · 工作线程池就绪',
      featuresTitle: '第一性原理架构',
      featuresSubtitle: '专为极致吞吐量、消除线程阻塞与企业级安全沙箱倾力打造。',
      features: [
        {
          title: '多 Worker 线程池并发',
          desc: 'HTTP 服务采用无锁 Channel 跨线程分发与工作线程池，彻底消除线程创建风暴与上下文切换瓶颈。',
        },
        {
          title: '全内存双重模块缓存',
          desc: '双层两级预解析 require 缓存，完全绕过磁盘 stat 系统调用，达成 4,601,226 ops/s 的解析速度。',
        },
        {
          title: '原生 TypeScript 6.0 & TSX',
          desc: '由 oxc 强力驱动：瞬时擦除类型，全面支持 Stage 3 装饰器、using 语法糖与 JSX 降级，无需 tsc 额外开销。',
        },
        {
          title: '默认闭合安全沙箱',
          desc: '通过 --sandbox 实现细粒度权限控制，目录隔离限制、运行期 JSONL 审计日志以及白名单放行机制。',
        },
        {
          title: '高精度优化时间轮',
          desc: '系统级优化事件循环定时器队列，1,000 个高频定时器仅需 2.51ms 即可高效触发（提速 14.2 倍）。',
        },
        {
          title: 'Node.js 与 Web 标准深度兼容',
          desc: '全面支持 node:http、node:buffer、node:crypto、fetch、WebSocket、WebCrypto 及 WebAssembly 原生 JIT。',
        },
      ],
      systemsTitle: '运行时核心子系统',
      systemsSubtitle: '基于 Rust 构建的模块化、超高性能系统架构。',
      systemsMeta: '架构蓝图',
      systemsLabel: '子系统',
      systems: [
        {
          title: 'V8 运行时与 Isolate 核心',
          desc: '深度集成 Google V8 C++ 绑定，带来原生级执行性能、轻量堆内存占用与 WASM JIT 支持。',
        },
        {
          title: 'oxc TypeScript 转译引擎',
          desc: '采用超高速 Rust AST 解析器，以亚毫秒级速度擦除类型并将现代 TS 语法降级为 ES2022。',
        },
        {
          title: '多 Worker 线程池并发系统',
          desc: '无锁任务调度线程池，零线程创建成本轻松支撑每秒数万并发 HTTP 连接。',
        },
        {
          title: 'Node.js 兼容层',
          desc: '深度实现 fs、path、crypto、buffer、events、http、process、timers 及 CommonJS require。',
        },
        {
          title: 'Web 标准 API 层',
          desc: '符合 W3C 标准的 fetch、URL、Streams、Blob、Web Crypto、BroadcastChannel 与 ServiceWorker 实现。',
        },
        {
          title: '免配置内置测试框架',
          desc: 'Jest 兼容的测试套件，内置断言、用例发现、并行执行与覆盖率分析。',
        },
      ],
      ctaTitle: '准备好体验极致性能了吗？',
      ctaSubtitle: '一行命令即可在 macOS 和 Linux 上安装 Beejs v0.4.3。',
      ctaButton: '查看安装手册',
      ctaNotesButton: '阅读技术发布日志',
    },
    docs: {
      title: '运行时手册',
      subtitle: 'Beejs v0.4.3 开发者与运维手册。',
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
            'Beejs v0.4.3 是运行时的重大性能突破版本。引入多 Worker 线程池并发、全内存双重模块缓存、极速冷启动与完备 Node/Web API 兼容。',
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
            '$ curl -fsSL https://bee.zhanghe.dev/install.sh | sh',
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
          subtitle: 'TS 和 TSX 文件执行前由 oxc 转译。',
          body: [
            '当 .ts 或 .tsx 文件传给 bee run 时，CLI 会走 oxc（TypeScript 6.0 语法，仅转译）。类型会被擦除。using 和 Stage 3 装饰器会降级到 ES2022，以适应当前 V8。TSX 输出 classic React.createElement。TypeScript 7.0 没有新语法。',
          ],
          list: [
            '可使用 .ts、.tsx、.mts、.cts、.jsx 入口文件',
            '这不是 tsc --noEmit。类型错误只要生成的 JS 合法仍可能执行',
            '未使用的 value import 会保留（可能有副作用），只擦除 import type',
            'bee run examples/basics/typescript_latest.ts',
          ],
        },
        'memory-management': {
          title: '兼容层',
          subtitle: '提供选定 Node.js 和 Web API。',
          body: ['默认构建包含常见 Node.js 与 Web API 兼容层。覆盖并非完整标准实现，依赖具体边界前应查看示例或测试。'],
          list: ['Node.js 模块包括 fs、path、crypto、buffer、process、timers 和 require', 'Web API 包括 fetch、URL、streams、Blob、events、timers 和部分 Web Crypto'],
        },
        'server-mode': {
          title: 'Serve 模式',
          subtitle: '健康检查 stub，不是应用服务器。',
          body: [
            'bee serve 用 tiny_http 绑定端口并返回固定 {"ok":true}，不执行用户脚本。应用 HTTP 请用 http.createServer 和 bee run。',
          ],
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
            'bee serve - 健康检查 stub（固定 JSON，不跑用户脚本）',
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
      title: '发布日志',
      subtitle: '运行时动态、工程实现更新与版本发布范围。',
      tagLabel: '主题',
      back: '返回发布日志',
      operator: '作者',
      by: '作者：',
      timestamp: '日期',
      readTime: '阅读时长',
      readMore: '阅读全文',
      notFound: '未找到相关文章',
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
