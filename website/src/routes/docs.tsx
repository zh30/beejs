import { Link, useParams } from 'react-router-dom'
import { Book, ChevronRight, Code, Terminal, Layers, Cpu, Zap, ArrowLeft } from 'lucide-react'
import { motion } from 'framer-motion'
import { BeeLogo } from '../components/Logo'

export default function DocsComponent() {
  const { section = 'introduction' } = useParams()

  const sections = [
    {
      title: 'Initialize',
      items: [
        { id: 'introduction', name: 'Introduction', icon: <Book className="w-4 h-4" /> },
        { id: 'installation', name: 'Installation', icon: <Terminal className="w-4 h-4" /> },
        { id: 'quick-start', name: 'Quick Start', icon: <Zap className="w-4 h-4" /> },
      ]
    },
    {
      title: 'Core Architecture',
      items: [
        { id: 'v8-isolate-pool', name: 'V8 Isolate Pool', icon: <Cpu className="w-4 h-4" /> },
        { id: 'jit-optimization', name: 'JIT Optimization', icon: <Zap className="w-4 h-4" /> },
        { id: 'memory-management', name: 'Memory Management', icon: <Layers className="w-4 h-4" /> },
      ]
    },
    {
      title: 'Manuals',
      items: [
        { id: 'cli-usage', name: 'CLI Usage', icon: <Code className="w-4 h-4" /> },
        { id: 'api-reference', name: 'API Reference', icon: <Book className="w-4 h-4" /> },
        { id: 'modules', name: 'Modules', icon: <Layers className="w-4 h-4" /> },
      ]
    }
  ]

  const renderContent = () => {
    switch (section) {
      case 'introduction':
        return <IntroductionContent />
      case 'installation':
        return <InstallationContent />
      case 'quick-start':
        return <QuickStartContent />
      case 'v8-isolate-pool':
        return <V8IsolatePoolContent />
      case 'jit-optimization':
        return <JITOptimizationContent />
      case 'memory-management':
        return <MemoryManagementContent />
      case 'cli-usage':
        return <CLIUsageContent />
      case 'api-reference':
        return <APIReferenceContent />
      case 'modules':
        return <ModulesContent />
      default:
        return <IntroductionContent />
    }
  }

  return (
    <div className="relative min-h-screen bg-brand-black">
      {/* Background Decor */}
      <div className="absolute inset-0 cyber-grid opacity-20 pointer-events-none" />
      <div className="scanline opacity-5 pointer-events-none" />

      <div className="max-w-7xl mx-auto flex flex-col md:flex-row py-24 px-4 md:px-8 gap-16 relative z-10">
        {/* Sidebar */}
        <aside className="w-full md:w-72 shrink-0">
          <nav className="sticky top-32 space-y-12">
            <div className="flex flex-col space-y-8 mb-12">
              <BeeLogo className="w-12 h-12" />
              <Link to="/" className="inline-flex items-center text-xs font-black text-gray-500 hover:text-brand-yellow transition-colors uppercase tracking-[0.2em]">
                <ArrowLeft className="w-3 h-3 mr-2" /> Return to Base
              </Link>
            </div>

            {sections.map((s) => (
              <div key={s.title}>
                <h3 className="text-[10px] font-black uppercase tracking-[0.4em] text-gray-600 mb-6 px-4 border-l-2 border-brand-yellow/20">
                  {s.title}
                </h3>
                <ul className="space-y-2">
                  {s.items.map((item) => (
                    <li key={item.id}>
                      <Link
                        to={`/docs/${item.id}`}
                        className={`w-full flex items-center space-x-3 px-4 py-3 transition-all group text-left -skew-x-12 ${section === item.id
                          ? 'bg-brand-yellow text-brand-black font-black'
                          : 'text-gray-400 hover:text-white hover:bg-white/5 border border-transparent'
                          }`}
                      >
                        <span className="skew-x-12 flex items-center space-x-3">
                          <span className={section === item.id ? 'text-brand-black' : 'text-brand-yellow opacity-50 group-hover:opacity-100'}>
                            {item.icon}
                          </span>
                          <span className="text-xs uppercase tracking-widest">{item.name}</span>
                        </span>
                      </Link>
                    </li>
                  ))}
                </ul>
              </div>
            ))}
          </nav>
        </aside>

        {/* Content */}
        <motion.main
          key={section}
          initial={{ opacity: 0, x: 20 }}
          animate={{ opacity: 1, x: 0 }}
          className="grow max-w-4xl"
        >
          {renderContent()}
        </motion.main>
      </div>
    </div>
  )
}

function SectionHeader({ title, subtitle }: { title: string, subtitle?: string }) {
  return (
    <header className="mb-16">
      <h1 className="text-4xl md:text-7xl font-black text-white uppercase italic tracking-tighter leading-none mb-6">
        {title.split(' ').map((word, i) => (
          <span key={i} className={i % 2 !== 0 ? 'text-brand-yellow glow-text' : ''}>
            {word}{' '}
          </span>
        ))}
      </h1>
      {subtitle && <p className="text-gray-400 text-lg md:text-xl font-mono uppercase tracking-widest">{subtitle}</p>}
      <div className="h-px w-full bg-linear-to-r from-brand-yellow/50 to-transparent mt-8" />
    </header>
  )
}

function CodeBlock({ code, lang = "bash" }: { code: string, lang?: string }) {
  return (
    <div className="relative group my-8">
      <div className="absolute -inset-0.5 bg-brand-yellow/20 rounded-lg blur opacity-30 group-hover:opacity-50 transition duration-1000"></div>
      <div className="relative bg-black/80 rounded-lg overflow-hidden border border-white/10">
        <div className="flex items-center justify-between px-4 py-2 border-b border-white/5 bg-white/5">
          <div className="flex space-x-1.5">
            <div className="w-2.5 h-2.5 rounded-full bg-red-500/50" />
            <div className="w-2.5 h-2.5 rounded-full bg-yellow-500/50" />
            <div className="w-2.5 h-2.5 rounded-full bg-green-500/50" />
          </div>
          <span className="text-[10px] font-mono text-gray-500 uppercase">{lang}</span>
        </div>
        <pre className="p-6 overflow-x-auto text-sm font-mono text-gray-300 leading-relaxed">
          <code>{code}</code>
        </pre>
      </div>
    </div>
  )
}

function IntroductionContent() {
  return (
    <div className="space-y-12 pb-24">
      <SectionHeader
        title="Introduction to Beejs"
        subtitle="The Next Generation JavaScript/TypeScript Engine"
      />

      <div className="prose prose-invert prose-brand max-w-none text-gray-400 leading-relaxed text-lg">
        <p>
          Beejs is a high-performance JavaScript/TypeScript runtime built with Rust and V8.
          It is engineered from the ground up for the **AI era**, focusing on extreme startup speed,
          sophisticated memory management, and native AI co-processing.
        </p>
      </div>

      <div className="grid grid-cols-1 md:grid-cols-2 gap-6 mt-12">
        <div className="glass p-8 neon-border-animated">
          <h3 className="text-xl font-black text-white uppercase italic mb-4 tracking-tighter">Extreme Startup</h3>
          <p className="text-sm text-gray-500 leading-relaxed font-mono">11ms. Cold start. No hydration overhead. The fastest way to hit your entry point.</p>
        </div>
        <div className="glass p-8 neon-border-animated">
          <h3 className="text-xl font-black text-white uppercase italic mb-4 tracking-tighter">AI Efficiency</h3>
          <p className="text-sm text-gray-500 leading-relaxed font-mono">19.6% reduction in memory overhead. Optimized for gigabyte-scale LLM workloads.</p>
        </div>
      </div>

      <div className="mt-20 flex justify-end">
        <Link to="/docs/installation" className="px-8 py-4 bg-white text-brand-black font-black uppercase -skew-x-12 hover:bg-brand-yellow transition-all">
          <span className="inline-block skew-x-12 flex items-center">
            Next: Installation <ChevronRight className="w-5 h-5 ml-2" />
          </span>
        </Link>
      </div>
    </div>
  )
}

function InstallationContent() {
  return (
    <div className="space-y-12 pb-24">
      <SectionHeader title="System Installation" subtitle="Initialize the Hive" />

      <section>
        <h2 className="text-2xl font-black text-white uppercase italic mb-6 tracking-tighter">Automated Script</h2>
        <p className="text-gray-400 mb-6">Compatible with macOS, Linux, and WSL2 environments. One command to rule them all.</p>
        <CodeBlock code="$ curl -fsSL https://beejs.zhanghe.dev/install.sh | sh" />
      </section>

      <section className="mt-20">
        <h2 className="text-2xl font-black text-white uppercase italic mb-6 tracking-tighter">Binary Download</h2>
        <p className="text-gray-400 mb-6">For air-gapped systems or custom pipelines, download the optimized binary for your target architecture.</p>
        <div className="glass p-8 border border-white/10 bg-white/5 space-y-4 font-mono text-sm leading-relaxed">
          <p className="text-brand-yellow font-black"># Example for Linux x86_64</p>
          <p>wget https://github.com/zh30/beejs/releases/latest/download/beejs-linux-x64.tar.gz</p>
          <p>tar -xzf beejs-linux-x64.tar.gz</p>
          <p>sudo mv beejs /usr/local/bin/</p>
        </div>
      </section>

      <div className="mt-20 flex justify-between">
        <Link to="/docs/introduction" className="text-gray-600 font-black uppercase tracking-widest hover:text-white transition-colors">
          ← Introduction
        </Link>
        <Link to="/docs/quick-start" className="px-8 py-4 bg-brand-yellow text-brand-black font-black uppercase -skew-x-12 hover:bg-white transition-all">
          <span className="inline-block skew-x-12 flex items-center">
            Next: Quick Start <ChevronRight className="w-5 h-5 ml-2" />
          </span>
        </Link>
      </div>
    </div>
  )
}

function QuickStartContent() {
  return (
    <div className="space-y-12 pb-24">
      <SectionHeader title="Quick Start" subtitle="Deployment in Microseconds" />

      <section>
        <h2 className="text-2xl font-black text-white uppercase italic mb-6 tracking-tighter">First Execution</h2>
        <p className="text-gray-400 mb-6">Create `hello.ts` and experience the speed instantly.</p>
        <CodeBlock lang="typescript" code={`const name: string = "Beejs Operator";\nconsole.log(\`Hello \${name} from the fastest runtime!\`);\n\n// Trigger Native AI Optimization\nBeejs.optimize('speed');`} />
        <CodeBlock code="$ beejs hello.ts" />
      </section>

      <div className="mt-20 flex justify-between">
        <Link to="/docs/installation" className="text-gray-600 font-black uppercase tracking-widest hover:text-white transition-colors">
          ← Installation
        </Link>
        <Link to="/docs/v8-isolate-pool" className="px-8 py-4 bg-brand-yellow text-brand-black font-black uppercase -skew-x-12 hover:bg-white transition-all">
          <span className="inline-block skew-x-12 flex items-center">
            Next: V8 Isolate Pool <ChevronRight className="w-5 h-5 ml-2" />
          </span>
        </Link>
      </div>
    </div>
  )
}

function V8IsolatePoolContent() {
  return (
    <div className="space-y-12 pb-24">
      <SectionHeader title="V8 Isolate Pool" subtitle="Architectural Superiority" />

      <div className="prose prose-invert prose-brand max-w-none text-gray-400 leading-relaxed text-lg">
        <p>
          The secret behind the 11ms startup is the **Warm Pool**. We don't hydrate V8 isolates from cold disk images;
          we maintain a fleet of pre-initialized environments ready to receive your bytecode.
        </p>
      </div>

      <div className="relative glass p-10 border border-brand-yellow/30 bg-brand-yellow/5">
        <div className="absolute top-0 right-0 px-3 py-1 bg-brand-yellow text-brand-black text-[10px] font-black uppercase">CORE TECH</div>
        <h4 className="text-brand-yellow font-black mb-6 uppercase tracking-widest">Hydration Workflow</h4>
        <ul className="space-y-4 font-mono text-sm text-gray-300">
          <li className="flex items-start gap-4"><span className="text-brand-yellow">[01]</span> Claim pre-initialized Isolate from the Rust pool.</li>
          <li className="flex items-start gap-4"><span className="text-brand-yellow">[02]</span> Fast-reset of heap pointers to snapshot base state.</li>
          <li className="flex items-start gap-4"><span className="text-brand-yellow">[03]</span> Instant execution of entry-point script.</li>
        </ul>
      </div>

      <div className="mt-20 flex justify-between">
        <Link to="/docs/quick-start" className="text-gray-600 font-black uppercase tracking-widest hover:text-white transition-colors">
          ← Quick Start
        </Link>
        <Link to="/docs/jit-optimization" className="px-8 py-4 bg-brand-yellow text-brand-black font-black uppercase -skew-x-12 hover:bg-white transition-all">
          <span className="inline-block skew-x-12 flex items-center">
            Next: JIT Optimization <ChevronRight className="w-5 h-5 ml-2" />
          </span>
        </Link>
      </div>
    </div>
  )
}

function JITOptimizationContent() {
  return (
    <div className="space-y-12 pb-24">
      <SectionHeader title="JIT Optimization" subtitle="Telemetry-Driven Compilation" />
      <p className="text-gray-400">Beejs uses real-time telemetry from the Tokio scheduler to adjust V8's optimization thresholds dynamically.</p>

      <div className="grid grid-cols-1 md:grid-cols-2 gap-1 bg-white/5 border border-white/10">
        <div className="p-8 border-b md:border-b-0 md:border-r border-white/10">
          <h4 className="text-white font-black uppercase italic mb-4">Turbo Path</h4>
          <p className="text-xs text-gray-500 leading-relaxed">Aggressive JIT triggers for detected hot functions, moving from bytecode to machine code in microseconds.</p>
        </div>
        <div className="p-8">
          <h4 className="text-white font-black uppercase italic mb-4">Smart Recovery</h4>
          <p className="text-xs text-gray-500 leading-relaxed">Immediate re-calibration when de-optimization signatures are detected in polymorphic logic.</p>
        </div>
      </div>

      <div className="mt-20 flex justify-between">
        <Link to="/docs/v8-isolate-pool" className="text-gray-600 font-black uppercase tracking-widest hover:text-white transition-colors">
          ← V8 Isolate Pool
        </Link>
        <Link to="/docs/memory-management" className="px-8 py-4 bg-brand-yellow text-brand-black font-black uppercase -skew-x-12 hover:bg-white transition-all">
          <span className="inline-block skew-x-12 flex items-center">
            Next: Memory <ChevronRight className="w-5 h-5 ml-2" />
          </span>
        </Link>
      </div>
    </div>
  )
}

function MemoryManagementContent() {
  return (
    <div className="space-y-12 pb-24">
      <SectionHeader title="Memory Control" subtitle="Zero-Copy Efficiency" />
      <p className="text-gray-400">Native memory pooling reduces allocation overhead by 19.6%.</p>

      <div className="space-y-8 mt-12">
        <div className="glass p-8 border-l-4 border-brand-yellow">
          <h4 className="text-white font-black uppercase tracking-widest mb-4">Smart Allocation</h4>
          <p className="text-sm text-gray-500 leading-relaxed">Predictive allocation for common AI workloads prevents heap fragmentation before it occurs.</p>
        </div>
        <div className="glass p-8 border-l-4 border-gray-700">
          <h4 className="text-white font-black uppercase tracking-widest mb-4">Zero-Copy I/O</h4>
          <p className="text-sm text-gray-500 leading-relaxed">Rust-to-V8 data transfers bypass the serialization bottleneck entirely.</p>
        </div>
      </div>

      <div className="mt-20 flex justify-between">
        <Link to="/docs/jit-optimization" className="text-gray-600 font-black uppercase tracking-widest hover:text-white transition-colors">
          ← JIT Optimization
        </Link>
        <Link to="/docs/cli-usage" className="px-8 py-4 bg-brand-yellow text-brand-black font-black uppercase -skew-x-12 hover:bg-white transition-all">
          <span className="inline-block skew-x-12 flex items-center">
            Next: CLI Usage <ChevronRight className="w-5 h-5 ml-2" />
          </span>
        </Link>
      </div>
    </div>
  )
}

function CLIUsageContent() {
  return (
    <div className="space-y-12 pb-24">
      <SectionHeader title="CLI Usage" subtitle="Operator Directives" />
      <div className="grid grid-cols-1 gap-4">
        {[
          { cmd: 'run <file>', desc: 'Optimize and execute target script.' },
          { cmd: '--watch', desc: 'Hot-load changes with zero downtime.' },
          { cmd: 'test', desc: 'Execute native performance test suite.' },
          { cmd: 'init', desc: 'Bootstrap a new high-performance hive.' },
        ].map(item => (
          <div key={item.cmd} className="flex flex-col md:flex-row md:items-center justify-between p-6 glass border border-white/5 hover:border-brand-yellow/30 transition-colors">
            <code className="text-brand-yellow font-black mb-2 md:mb-0 uppercase tracking-widest">beejs {item.cmd}</code>
            <span className="text-xs text-gray-500 uppercase tracking-widest italic">{item.desc}</span>
          </div>
        ))}
      </div>

      <div className="mt-20 flex justify-between">
        <Link to="/docs/memory-management" className="text-gray-600 font-black uppercase tracking-widest hover:text-white transition-colors">
          ← Memory
        </Link>
        <Link to="/docs/api-reference" className="px-8 py-4 bg-brand-yellow text-brand-black font-black uppercase -skew-x-12 hover:bg-white transition-all">
          <span className="inline-block skew-x-12 flex items-center">
            Next: APIs <ChevronRight className="w-5 h-5 ml-2" />
          </span>
        </Link>
      </div>
    </div>
  )
}

function APIReferenceContent() {
  return (
    <div className="space-y-12 pb-24">
      <SectionHeader title="API Reference" subtitle="Core Intermediary" />
      <div className="space-y-8">
        <div className="p-10 bg-linear-to-br from-white/5 to-transparent border border-white/5">
          <code className="text-brand-yellow text-xl font-black uppercase italic tracking-tighter block mb-6">Beejs.optimize(mode)</code>
          <p className="text-gray-400 mb-8 leading-relaxed">Direct runtime to prioritize specific performance profiles.</p>
          <div className="flex flex-wrap gap-3">
            {['speed', 'size', 'auto'].map(m => (
              <span key={m} className="px-3 py-1 bg-white/10 text-white font-mono text-[10px] uppercase">{m}</span>
            ))}
          </div>
        </div>
      </div>

      <div className="mt-20 flex justify-between">
        <Link to="/docs/cli-usage" className="text-gray-600 font-black uppercase tracking-widest hover:text-white transition-colors">
          ← CLI Usage
        </Link>
        <Link to="/docs/modules" className="px-8 py-4 bg-brand-yellow text-brand-black font-black uppercase -skew-x-12 hover:bg-white transition-all">
          <span className="inline-block skew-x-12 flex items-center">
            Next: Modules <ChevronRight className="w-5 h-5 ml-2" />
          </span>
        </Link>
      </div>
    </div>
  )
}

function ModulesContent() {
  return (
    <div className="space-y-12 pb-24">
      <SectionHeader title="Native Modules" subtitle="Rust Accelerated Core" />
      <div className="grid grid-cols-1 md:grid-cols-3 gap-6">
        {[
          { name: 'beejs:fs', desc: 'Tokio-backed non-blocking file I/O.' },
          { name: 'beejs:net', desc: 'Ultra-low latency network layer.' },
          { name: 'beejs:ai', desc: 'Hardware-bound inference engine.' },
        ].map(mod => (
          <div key={mod.name} className="glass p-8 border border-white/10 hover:border-brand-yellow transition-all flex flex-col justify-between">
            <code className="text-brand-yellow font-black mb-4 group-hover:glow-text uppercase">{mod.name}</code>
            <p className="text-[10px] text-gray-500 uppercase tracking-widest leading-loose">{mod.desc}</p>
          </div>
        ))}
      </div>

      <div className="mt-20 flex justify-between">
        <Link to="/docs/api-reference" className="text-gray-600 font-black uppercase tracking-widest hover:text-white transition-colors">
          ← APIs
        </Link>
        <Link to="/blog" className="px-10 py-5 bg-white text-brand-black font-black uppercase -skew-x-12 hover:bg-brand-yellow transition-all">
          <span className="inline-block skew-x-12">Read Terminal</span>
        </Link>
      </div>
    </div>
  )
}
