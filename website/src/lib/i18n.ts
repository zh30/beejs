import React, { createContext, useContext, useEffect, useMemo, useState } from 'react'

export type Lang = 'en' | 'zh'

type LangContextValue = {
  lang: Lang
  setLang: (lang: Lang) => void
  toggle: () => void
  copy: typeof copy.en
}

const copy = {
  en: {
    nav: {
      home: 'Home',
      docs: 'Manual',
      blog: 'Flight Log',
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
      stage: 'Stage 93',
      contact: 'Comms',
      email: 'support@beejs.zhanghe.dev',
      rights: 'All rights reserved.',
    },
    home: {
      kicker: 'Bridge Console',
      title: 'Beejs Runtime',
      titleAccent: 'Performance',
      subtitle:
        'AI-native JavaScript/TypeScript runtime built with Rust + V8. 11ms cold start, 100M ops/sec throughput.',
      ctaPrimary: 'Open Manual',
      ctaSecondary: 'Read Flight Log',
      heroMetricLabel: 'Peak Throughput',
      heroMetricValue: '100M',
      heroMetricUnit: 'ops/sec',
      heroMetricNote: 'Simple arithmetic benchmark',
      heroFootnote: 'Beejs Performance Stack',
      telemetryTitle: 'Telemetry',
      telemetrySubtitle: 'Verified across 10,000+ cycles.',
      telemetryNote: 'AI runtime metrics validated with automated regression checks.',
      telemetry: [
        { label: 'Cold Start', value: '11ms', delta: '-84%', note: 'vs Node' },
        { label: 'Isolate Heap', value: '82MB', delta: '-19.6%', note: 'AI workload' },
        { label: 'Concurrency', value: '11.2K', delta: '+36.6%', note: 'parallel ops' },
        { label: 'I/O Latency', value: '0-copy', delta: 'x50', note: 'stream path' },
      ],
      benchmarksTitle: 'Benchmarks',
      benchmarksSubtitle: 'Normalized against Bun and Node.',
      benchmarksMeta: 'Telemetry index',
      benchmarks: [
        { label: 'Arithmetic', value: '100M', unit: 'ops/sec', delta: '+102,404%' },
        { label: 'String Ops', value: '33M', unit: 'ops/sec', delta: '+170,728%' },
        { label: 'Object Ops', value: '20M', unit: 'ops/sec', delta: '+1,375,510%' },
      ],
      systemsTitle: 'Core Systems',
      systemsSubtitle: 'Minimal surface, maximal throughput.',
      systemsMeta: 'Subsystem map',
      systemsLabel: 'module',
      systems: [
        {
          title: 'Turbo JIT',
          desc: 'Adaptive compilation thresholds tuned by telemetry feedback loops.',
        },
        {
          title: 'Zero-Copy I/O',
          desc: 'Rust-to-V8 transfers bypass serialization bottlenecks entirely.',
        },
        {
          title: 'Server Mode',
          desc: 'Persistent isolates for low-latency HTTP and WebSocket execution.',
        },
        {
          title: 'AI Co-Processor',
          desc: 'Batch + tensor acceleration designed for inference workloads.',
        },
        {
          title: 'Smart Cache',
          desc: 'Multi-tier cache with predictive prefetch for hot modules.',
        },
        {
          title: 'Core Observer',
          desc: 'Microsecond telemetry and automatic regression detection.',
        },
      ],
      ctaTitle: 'Ready to Deploy?',
      ctaSubtitle: 'Open source, Rust core, AI-optimized runtime.',
      ctaButton: 'Initialize',
    },
    docs: {
      title: 'Ship Manual',
      subtitle: 'Operator-grade documentation for Beejs runtime.',
      backToHome: 'Return to Bridge',
      groups: [
        {
          title: 'Initialize',
          items: [
            { id: 'introduction', label: 'Overview' },
            { id: 'installation', label: 'Installation' },
            { id: 'quick-start', label: 'Quick Start' },
          ],
        },
        {
          title: 'Core Architecture',
          items: [
            { id: 'v8-isolate-pool', label: 'V8 Isolate Pool' },
            { id: 'jit-optimization', label: 'JIT Optimization' },
            { id: 'memory-management', label: 'Memory Control' },
            { id: 'server-mode', label: 'Server Mode' },
          ],
        },
        {
          title: 'Operations',
          items: [
            { id: 'cli-usage', label: 'CLI Usage' },
            { id: 'api-reference', label: 'API Surface' },
            { id: 'modules', label: 'Native Modules' },
          ],
        },
      ],
      sections: {
        introduction: {
          title: 'Introduction',
          subtitle: 'Rust + V8 runtime for AI-grade workloads.',
          body: [
            'Beejs is engineered for extreme startup speed and sustained throughput. It pairs Rust systems control with deep V8 optimization for the AI era.',
            'Server Mode keeps isolates warm to eliminate cold-start penalties for repeated workloads.',
          ],
          cards: [
            { title: '11ms Startup', desc: 'Cold boot to execution in milliseconds.' },
            { title: '100M ops/sec', desc: 'Peak arithmetic throughput in core benchmarks.' },
          ],
        },
        installation: {
          title: 'Installation',
          subtitle: 'Bootstrap the runtime in minutes.',
          body: [
            'Recommended for macOS, Linux, and WSL2. Use the installer or fetch the binary directly for air-gapped systems.',
          ],
          code: [
            '$ curl -fsSL https://beejs.zhanghe.dev/install.sh | sh',
            '$ beejs --version',
          ],
        },
        'quick-start': {
          title: 'Quick Start',
          subtitle: 'Run your first script.',
          body: ['Create a file and run it instantly.'],
          code: [
            'console.log("Hello from Beejs");',
            'beejs run hello.js',
          ],
        },
        'v8-isolate-pool': {
          title: 'V8 Isolate Pool',
          subtitle: 'Warm pools keep latency low.',
          body: [
            'A fleet of pre-initialized isolates is kept warm so bytecode can execute immediately without full hydration.',
          ],
          list: [
            'Claim isolate from the warm pool',
            'Reset heap pointers to baseline snapshot',
            'Execute entrypoint in microseconds',
          ],
        },
        'jit-optimization': {
          title: 'JIT Optimization',
          subtitle: 'Telemetry-driven compilation.',
          body: [
            'Runtime instrumentation shifts compilation thresholds based on real workload signals.',
          ],
          list: [
            'Hot-path detection for aggressive inlining',
            'Fast recovery on de-optimization signals',
          ],
        },
        'memory-management': {
          title: 'Memory Control',
          subtitle: 'Zero-copy efficiency at scale.',
          body: [
            'Predictive allocation reduces fragmentation while zero-copy I/O keeps data transfers lean.',
          ],
          list: [
            'Adaptive GC thresholds',
            '19.6% reduction in overhead for AI workloads',
          ],
        },
        'server-mode': {
          title: 'Server Mode',
          subtitle: 'Persistent execution layer.',
          body: [
            'Beejs switches from one-shot execution to a long-running service with warm isolates and shared state.',
          ],
          code: ['$ beejs server --host 0.0.0.0 --port 3000'],
        },
        'cli-usage': {
          title: 'CLI Usage',
          subtitle: 'Operator directives.',
          list: [
            'beejs run <file> — execute scripts',
            'beejs server — start persistent runtime',
            'beejs test — run performance suites',
            'beejs repl — interactive shell',
          ],
        },
        'api-reference': {
          title: 'API Surface',
          subtitle: 'Core runtime commands.',
          body: ['Expose runtime configuration for tuning.'],
          list: ['Beejs.optimize("speed" | "size" | "auto")', 'Beejs.monitor.enable()'],
        },
        modules: {
          title: 'Native Modules',
          subtitle: 'Rust-accelerated primitives.',
          list: ['beejs:fs — non-blocking file I/O', 'beejs:net — low-latency network stack', 'beejs:ai — inference helpers'],
        },
      },
    },
    blog: {
      title: 'Flight Log',
      subtitle: 'Telemetry, internals, and release notes.',
      tagLabel: 'Manifest',
      back: 'Return to Flight Log',
      operator: 'Operator',
      timestamp: 'Timestamp',
      readTime: 'Sync Time',
      readMore: 'Open Log',
      notFound: 'Post Not Found',
    },
  },
  zh: {
    nav: {
      home: '首页',
      docs: '手册',
      blog: '航行日志',
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
      stage: 'Stage 93',
      contact: '通讯',
      email: 'support@beejs.zhanghe.dev',
      rights: '保留所有权利。',
    },
    home: {
      kicker: '舰桥控制台',
      title: 'Beejs 运行时',
      titleAccent: '性能',
      subtitle: 'Rust + V8 构建的 AI 原生运行时。11ms 冷启动，100M ops/sec 吞吐。',
      ctaPrimary: '打开手册',
      ctaSecondary: '查看航行日志',
      heroMetricLabel: '峰值吞吐',
      heroMetricValue: '100M',
      heroMetricUnit: 'ops/sec',
      heroMetricNote: '简单算术基准',
      heroFootnote: 'Beejs 性能栈',
      telemetryTitle: '遥测',
      telemetrySubtitle: '10,000+ 次验证循环。',
      telemetryNote: '自动化回归检测验证 AI 运行时指标。',
      telemetry: [
        { label: '冷启动', value: '11ms', delta: '-84%', note: '对比 Node' },
        { label: 'Isolate 堆', value: '82MB', delta: '-19.6%', note: 'AI 负载' },
        { label: '并发量', value: '11.2K', delta: '+36.6%', note: '并行任务' },
        { label: 'I/O 延迟', value: '0-copy', delta: 'x50', note: '流式路径' },
      ],
      benchmarksTitle: '性能基准',
      benchmarksSubtitle: '对比 Bun 与 Node 的标准化结果。',
      benchmarksMeta: '遥测索引',
      benchmarks: [
        { label: '算术', value: '100M', unit: 'ops/sec', delta: '+102,404%' },
        { label: '字符串', value: '33M', unit: 'ops/sec', delta: '+170,728%' },
        { label: '对象操作', value: '20M', unit: 'ops/sec', delta: '+1,375,510%' },
      ],
      systemsTitle: '核心系统',
      systemsSubtitle: '极简接口，极致吞吐。',
      systemsMeta: '子系统地图',
      systemsLabel: '模块',
      systems: [
        {
          title: 'Turbo JIT',
          desc: '基于遥测反馈动态调整编译阈值。',
        },
        {
          title: '零拷贝 I/O',
          desc: 'Rust 与 V8 之间传输无需序列化。',
        },
        {
          title: 'Server Mode',
          desc: '持久化 Isolate，HTTP/WS 低延迟执行。',
        },
        {
          title: 'AI 协处理',
          desc: '批处理与张量加速面向推理负载。',
        },
        {
          title: '智能缓存',
          desc: '多级缓存 + 预测预取提升热模块响应。',
        },
        {
          title: '核心观测',
          desc: '微秒级遥测与性能回归监测。',
        },
      ],
      ctaTitle: '准备部署？',
      ctaSubtitle: '开源，Rust 内核，面向 AI 优化。',
      ctaButton: '初始化',
    },
    docs: {
      title: '舰船手册',
      subtitle: '面向操作员的 Beejs 运行时指南。',
      backToHome: '返回舰桥',
      groups: [
        {
          title: '初始化',
          items: [
            { id: 'introduction', label: '概览' },
            { id: 'installation', label: '安装' },
            { id: 'quick-start', label: '快速开始' },
          ],
        },
        {
          title: '核心架构',
          items: [
            { id: 'v8-isolate-pool', label: 'V8 Isolate 池' },
            { id: 'jit-optimization', label: 'JIT 优化' },
            { id: 'memory-management', label: '内存控制' },
            { id: 'server-mode', label: 'Server Mode' },
          ],
        },
        {
          title: '运行维护',
          items: [
            { id: 'cli-usage', label: 'CLI 用法' },
            { id: 'api-reference', label: 'API 接口' },
            { id: 'modules', label: '原生模块' },
          ],
        },
      ],
      sections: {
        introduction: {
          title: '概览',
          subtitle: '面向 AI 负载的 Rust + V8 运行时。',
          body: [
            'Beejs 以极致冷启动速度与持续吞吐为目标设计，面向 AI 时代的 JavaScript/TypeScript 执行层。',
            'Server Mode 维持 Isolate 热池，避免重复冷启动成本。',
          ],
          cards: [
            { title: '11ms 冷启动', desc: '毫秒级启动直达执行。' },
            { title: '100M ops/sec', desc: '核心算术基准峰值吞吐。' },
          ],
        },
        installation: {
          title: '安装',
          subtitle: '几分钟完成部署。',
          body: ['建议用于 macOS、Linux 与 WSL2。亦可直接下载二进制。'],
          code: [
            '$ curl -fsSL https://beejs.zhanghe.dev/install.sh | sh',
            '$ beejs --version',
          ],
        },
        'quick-start': {
          title: '快速开始',
          subtitle: '运行第一段脚本。',
          body: ['创建文件并立即执行。'],
          code: ['console.log("Hello from Beejs");', 'beejs run hello.js'],
        },
        'v8-isolate-pool': {
          title: 'V8 Isolate 池',
          subtitle: '热池让延迟更低。',
          body: ['预初始化 Isolate 队列保持热启动，字节码可即时执行。'],
          list: ['从热池领取 Isolate', '重置堆指针至快照基线', '微秒级执行入口脚本'],
        },
        'jit-optimization': {
          title: 'JIT 优化',
          subtitle: '基于遥测的编译策略。',
          body: ['运行时检测热点路径并动态调整编译阈值。'],
          list: ['热点函数快速内联', '反优化信号即刻修正'],
        },
        'memory-management': {
          title: '内存控制',
          subtitle: '零拷贝效率。',
          body: ['预测性分配降低碎片，零拷贝 I/O 保持传输轻量。'],
          list: ['自适应 GC 阈值', 'AI 负载减少 19.6% 额外开销'],
        },
        'server-mode': {
          title: 'Server Mode',
          subtitle: '持久化执行层。',
          body: ['Beejs 从一次性执行切换为常驻服务，维持 Isolate 热态。'],
          code: ['$ beejs server --host 0.0.0.0 --port 3000'],
        },
        'cli-usage': {
          title: 'CLI 用法',
          subtitle: '操作指令。',
          list: [
            'beejs run <file> — 执行脚本',
            'beejs server — 启动持久运行时',
            'beejs test — 运行性能套件',
            'beejs repl — 交互式终端',
          ],
        },
        'api-reference': {
          title: 'API 接口',
          subtitle: '核心运行时命令。',
          body: ['对运行时进行性能调优与观测。'],
          list: ['Beejs.optimize("speed" | "size" | "auto")', 'Beejs.monitor.enable()'],
        },
        modules: {
          title: '原生模块',
          subtitle: 'Rust 加速原语。',
          list: ['beejs:fs — 非阻塞文件 I/O', 'beejs:net — 低延迟网络栈', 'beejs:ai — 推理工具集'],
        },
      },
    },
    blog: {
      title: '航行日志',
      subtitle: '遥测、内部实现与发布记录。',
      tagLabel: '清单',
      back: '返回航行日志',
      operator: '操作员',
      timestamp: '时间戳',
      readTime: '同步耗时',
      readMore: '打开日志',
      notFound: '未找到日志',
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
