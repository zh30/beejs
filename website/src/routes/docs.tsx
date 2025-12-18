import { Link, useParams } from 'react-router-dom'
import { Book, ChevronRight, Code, Terminal, Layers, Cpu, Zap } from 'lucide-react'

export default function DocsComponent() {
  const { section = 'introduction' } = useParams()

  const sections = [
    {
      title: 'Getting Started',
      items: [
        { id: 'introduction', name: 'Introduction', icon: <Book className="w-4 h-4" /> },
        { id: 'installation', name: 'Installation', icon: <Terminal className="w-4 h-4" /> },
        { id: 'quick-start', name: 'Quick Start', icon: <Zap className="w-4 h-4" /> },
      ]
    },
    {
      title: 'Core Concepts',
      items: [
        { id: 'v8-isolate-pool', name: 'V8 Isolate Pool', icon: <Cpu className="w-4 h-4" /> },
        { id: 'jit-optimization', name: 'JIT Optimization', icon: <Zap className="w-4 h-4" /> },
        { id: 'memory-management', name: 'Memory Management', icon: <Layers className="w-4 h-4" /> },
      ]
    },
    {
      title: 'Reference',
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
    <div className="max-w-7xl mx-auto flex flex-col md:flex-row py-12 px-4 md:px-8 gap-12">
      {/* Sidebar */}
      <aside className="w-full md:w-64 shrink-0">
        <nav className="sticky top-24 space-y-8">
          {sections.map((s) => (
            <div key={s.title}>
              <h3 className="text-xs font-bold uppercase tracking-widest text-gray-500 mb-4 px-2">
                {s.title}
              </h3>
              <ul className="space-y-1">
                {s.items.map((item) => (
                  <li key={item.id}>
                    <Link
                      to={`/docs/${item.id}`}
                      className={`w-full flex items-center space-x-3 px-3 py-2 rounded-lg text-sm transition-colors group text-left ${section === item.id ? 'bg-brand-yellow/10 text-brand-yellow' : 'text-gray-400 hover:text-white hover:bg-white/5'
                        }`}
                    >
                      <span className={`${section === item.id ? 'text-brand-yellow' : 'text-gray-600 group-hover:text-brand-yellow'} transition-colors`}>
                        {item.icon}
                      </span>
                      <span>{item.name}</span>
                    </Link>
                  </li>
                ))}
              </ul>
            </div>
          ))}
        </nav>
      </aside>

      {/* Content */}
      <main className="grow max-w-3xl prose prose-invert prose-brand text-gray-300">
        {renderContent()}
      </main>
    </div>
  )
}

function IntroductionContent() {
  return (
    <div className="space-y-8">
      <div>
        <h1 className="text-4xl font-bold mb-4 text-white">Introduction to Beejs</h1>
        <p className="text-xl text-gray-400 leading-relaxed">
          Beejs is a high-performance JavaScript/TypeScript runtime built with Rust and V8.
          It's designed to be the fastest engine for executing server-side scripts,
          optimized specifically for the demands of the AI era.
        </p>
      </div>

      <section className="space-y-6">
        <h2 className="text-2xl font-bold border-b border-white/10 pb-2 text-white">The AI Era Runtime</h2>
        <p>
          As AI tasks become more prevalent, the need for a runtime that can handle massive concurrency
          with minimal overhead is paramount. Beejs addresses this by combining the raw performance
          of Rust with the industry-standard execution of Google's V8 engine.
        </p>
        <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
          <div className="glass p-6 rounded-xl border border-white/5">
            <h3 className="font-bold text-brand-yellow mb-2">Extreme Startup Speed</h3>
            <p className="text-sm text-gray-400">11ms cold start ensures your serverless functions and AI agents respond instantly.</p>
          </div>
          <div className="glass p-6 rounded-xl border border-white/5">
            <h3 className="font-bold text-brand-yellow mb-2">Memory Efficiency</h3>
            <p className="text-sm text-gray-400">Optimized memory pooling reduces overhead by 19.6% compared to other modern runtimes.</p>
          </div>
        </div>
      </section>

      <div className="mt-20 pt-8 border-t border-white/10 flex justify-end">
        <Link to="/docs/installation" className="text-brand-yellow hover:underline flex items-center space-x-2">
          <span>Installation</span>
          <ChevronRight className="w-4 h-4 ml-1" />
        </Link>
      </div>
    </div>
  )
}

function InstallationContent() {
  return (
    <div className="space-y-8">
      <h1 className="text-4xl font-bold mb-4 text-white">Installation</h1>
      <p className="text-gray-400">Beejs can be installed on macOS, Linux, and Windows via our automated script or manual binary download.</p>

      <section className="space-y-6">
        <h2 className="text-2xl font-bold border-b border-white/10 pb-2 text-white">Using the Install Script</h2>
        <div className="glass rounded-xl p-4 bg-black/40 font-mono text-sm">
          <code className="text-brand-yellow">
            $ curl -fsSL https://beejs.zhanghe.dev/install.sh | sh
          </code>
        </div>
        <p className="text-sm text-gray-500 italic">Compatible with macOS, Linux, and WSL.</p>
      </section>

      <section className="space-y-6">
        <h2 className="text-2xl font-bold border-b border-white/10 pb-2 text-white">Manual Installation</h2>
        <p>Download the pre-compiled binary for your architecture from the <a href="https://github.com/zh30/beejs/releases" className="text-brand-yellow hover:underline">GitHub Releases</a> page.</p>
        <div className="glass p-6 rounded-xl border border-white/10 bg-white/5">
          <h4 className="font-bold mb-3">Example for x86_64 Linux:</h4>
          <pre className="text-xs text-gray-400 leading-relaxed font-mono">
            {`wget https://github.com/zh30/beejs/releases/latest/download/beejs-x86_64.tar.gz
tar -xzf beejs-x86_64.tar.gz
chmod +x beejs
sudo mv beejs /usr/local/bin/`}
          </pre>
        </div>
      </section>

      <div className="mt-20 pt-8 border-t border-white/10 flex justify-between">
        <Link to="/docs/introduction" className="text-gray-500 hover:text-white">← Introduction</Link>
        <Link to="/docs/quick-start" className="text-brand-yellow hover:underline flex items-center space-x-2">
          <span>Quick Start</span>
          <ChevronRight className="w-4 h-4 ml-1" />
        </Link>
      </div>
    </div>
  )
}

function QuickStartContent() {
  return (
    <div className="space-y-8">
      <h1 className="text-4xl font-bold mb-4 text-white">Quick Start</h1>
      <p className="text-gray-400">Get up and running with your first Beejs script in seconds.</p>

      <section className="space-y-6">
        <h2 className="text-2xl font-bold border-b border-white/10 pb-2 text-white">Hello World</h2>
        <p>Create a file named <code>hello.ts</code>:</p>
        <div className="glass rounded-xl p-4 bg-black/40 font-mono text-sm">
          <pre className="text-gray-300">
            {`const name: string = "Beejs User";
console.log(\`Hello \${name} from the fastest runtime!\`);

// Native AI optimization
Beejs.optimize('speed');`}
          </pre>
        </div>
        <p>Run it with the <code>beejs</code> command:</p>
        <div className="glass p-4 rounded-xl bg-black/40 font-mono text-brand-yellow">
          $ beejs hello.ts
        </div>
      </section>

      <section className="space-y-6">
        <h2 className="text-2xl font-bold border-b border-white/10 pb-2 text-white">Project Initialization</h2>
        <p>Beejs comes with a built-in initializer to set up new projects quickly.</p>
        <div className="glass p-4 rounded-xl bg-black/40 font-mono text-brand-yellow">
          $ beejs init my-app
        </div>
      </section>

      <div className="mt-20 pt-8 border-t border-white/10 flex justify-between">
        <Link to="/docs/installation" className="text-gray-500 hover:text-white">← Installation</Link>
        <Link to="/docs/v8-isolate-pool" className="text-brand-yellow hover:underline flex items-center space-x-2">
          <span>V8 Isolate Pool</span>
          <ChevronRight className="w-4 h-4 ml-1" />
        </Link>
      </div>
    </div>
  )
}

function V8IsolatePoolContent() {
  return (
    <div className="space-y-8">
      <h1 className="text-4xl font-bold mb-4 text-white">V8 Isolate Pooling</h1>
      <div className="flex items-center space-x-2 text-brand-yellow bg-brand-yellow/10 px-3 py-1 rounded-full w-fit text-xs font-bold uppercase">
        <Cpu className="w-3 h-3" />
        <span>Performance Hero</span>
      </div>
      <p className="text-xl text-gray-400 leading-relaxed">
        The core of Beejs' 11ms startup time is its intelligent V8 Isolate Pooling system.
      </p>

      <section className="space-y-4">
        <h2 className="text-2xl font-bold text-white">How it Works</h2>
        <p>
          Traditional runtimes create a new V8 Isolate from scratch for every script execution,
          which involves significant overhead (snapshot hydration, heap allocation).
        </p>
        <p>
          Beejs maintains a **warm pool** of pre-initialized V8 Isolates. When a script needs to run:
        </p>
        <ul className="list-decimal list-inside space-y-2 text-gray-300 ml-4">
          <li>An available Isolate is claimed from the pool.</li>
          <li>The environment state is reset to a clean snapshot.</li>
          <li>Script execution begins instantly.</li>
        </ul>
        <div className="p-6 bg-brand-yellow/5 border border-brand-yellow/20 rounded-xl">
          <p className="text-brand-yellow font-bold mb-2">Result: 86% Faster Startups</p>
          <p className="text-sm text-gray-400">By reusing the V8 instance state, we bypass the most expensive parts of the runtime lifecycle.</p>
        </div>
      </section>

      <div className="mt-20 pt-8 border-t border-white/10 flex justify-between">
        <Link to="/docs/quick-start" className="text-gray-500 hover:text-white">← Quick Start</Link>
        <Link to="/docs/jit-optimization" className="text-brand-yellow hover:underline flex items-center space-x-2">
          <span>JIT Optimization</span>
          <ChevronRight className="w-4 h-4 ml-1" />
        </Link>
      </div>
    </div>
  )
}

function JITOptimizationContent() {
  return (
    <div className="space-y-8">
      <h1 className="text-4xl font-bold mb-4 text-white">Smart JIT Optimization</h1>
      <p className="text-gray-400 text-lg">
        Beejs dynamically adjusts JIT (Just-In-Time) compilation thresholds based on real-time execution telemetry.
      </p>

      <section className="space-y-4">
        <h2 className="text-2xl font-bold text-white">Dynamic Thresholding</h2>
        <p>
          Unlike static engines that wait for a fixed number of executions before optimizing a function,
          Beejs monitors **Hot Paths** and context complexity.
        </p>
        <div className="grid grid-cols-1 gap-4">
          <div className="glass p-6 rounded-xl">
            <h4 className="font-bold text-white mb-2 underline decoration-brand-yellow decoration-2">Aggressive Optimization</h4>
            <p className="text-sm text-gray-400 leading-relaxed">For detected bottlenecks, Beejs kicks in the TurboFan compiler earlier than standard V8 distributions.</p>
          </div>
          <div className="glass p-6 rounded-xl">
            <h4 className="font-bold text-white mb-2 underline decoration-brand-yellow decoration-2">De-optimization Recovery</h4>
            <p className="text-sm text-gray-400 leading-relaxed">Smart detection of "unfortunate" de-optimizations in polymorphic code, with immediate JIT re-calibration.</p>
          </div>
        </div>
      </section>

      <div className="mt-20 pt-8 border-t border-white/10 flex justify-between">
        <Link to="/docs/v8-isolate-pool" className="text-gray-500 hover:text-white">← V8 Isolate Pool</Link>
        <Link to="/docs/memory-management" className="text-brand-yellow hover:underline flex items-center space-x-2">
          <span>Memory Management</span>
          <ChevronRight className="w-4 h-4 ml-1" />
        </Link>
      </div>
    </div>
  )
}

function MemoryManagementContent() {
  return (
    <div className="space-y-8">
      <h1 className="text-4xl font-bold mb-4 text-white">Memory Management</h1>
      <p className="text-gray-400">Beejs utilizes a Smart Memory Pool to achieve a 19.6% reduction in memory usage compared to Bun.</p>

      <section className="space-y-6">
        <h2 className="text-2xl font-bold border-b border-white/10 pb-2 text-white">Zero-Copy I/O</h2>
        <p>
          Data is moved between the Rust core and the V8 heap using a zero-copy mechanism.
          This eliminates the performance hit and memory churn caused by serializing/deserializing data buffers.
        </p>
      </section>

      <section className="space-y-6">
        <h2 className="text-2xl font-bold border-b border-white/10 pb-2 text-white">Smart Allocation</h2>
        <p>
          Beejs pre-allocates memory chunks for common AI workloads, preventing heap fragmentation
          and ensuring predictable performance under high load.
        </p>
      </section>

      <div className="mt-20 pt-8 border-t border-white/10 flex justify-between">
        <Link to="/docs/jit-optimization" className="text-gray-500 hover:text-white">← JIT Optimization</Link>
        <Link to="/docs/cli-usage" className="text-brand-yellow hover:underline flex items-center space-x-2">
          <span>CLI Usage</span>
          <ChevronRight className="w-4 h-4 ml-1" />
        </Link>
      </div>
    </div>
  )
}

function CLIUsageContent() {
  return (
    <div className="space-y-8">
      <h1 className="text-4xl font-bold mb-4 text-white">CLI Usage</h1>
      <p className="text-gray-400 font-mono text-sm">The Swiss Army knife for JS/TS development.</p>

      <section className="space-y-6">
        <h2 className="text-2xl font-bold border-b border-white/10 pb-2 text-white">Core Commands</h2>
        <div className="space-y-4">
          <div className="glass p-4 rounded-xl flex items-start space-x-4">
            <code className="text-brand-yellow min-w-35">beejs run &lt;file&gt;</code>
            <p className="text-sm text-gray-500">Execute a JS or TS file with full optimization.</p>
          </div>
          <div className="glass p-4 rounded-xl flex items-start space-x-4">
            <code className="text-brand-yellow min-w-35">beejs --watch</code>
            <p className="text-sm text-gray-500">Run file with hot-reloading on changes.</p>
          </div>
          <div className="glass p-4 rounded-xl flex items-start space-x-4">
            <code className="text-brand-yellow min-w-35">beejs test</code>
            <p className="text-sm text-gray-500">Execute the built-in Jest-style test runner.</p>
          </div>
        </div>
      </section>

      <section className="space-y-6">
        <h2 className="text-2xl font-bold border-b border-white/10 pb-2 text-white">Package Manager</h2>
        <p>Beejs is fully compatible with <code>npm</code> and <code>yarn</code> packages.</p>
        <div className="glass p-4 rounded-xl font-mono text-brand-yellow">
          $ beejs install <br />
          $ beejs add lodash
        </div>
      </section>

      <div className="mt-20 pt-8 border-t border-white/10 flex justify-between">
        <Link to="/docs/memory-management" className="text-gray-500 hover:text-white">← Memory Management</Link>
        <Link to="/docs/api-reference" className="text-brand-yellow hover:underline flex items-center space-x-2">
          <span>API Reference</span>
          <ChevronRight className="w-4 h-4 ml-1" />
        </Link>
      </div>
    </div>
  )
}

function APIReferenceContent() {
  return (
    <div className="space-y-8">
      <h1 className="text-4xl font-bold mb-4 text-white">API Reference</h1>
      <p className="text-gray-400">Beejs provides a native global object with high-performance utilities.</p>

      <section className="space-y-6">
        <h2 className="text-2xl font-bold border-b border-white/10 pb-2 text-white">Global Object</h2>
        <div className="space-y-4">
          <div className="glass p-6 rounded-xl">
            <h3 className="text-brand-yellow font-mono mb-2">Beejs.optimize(mode)</h3>
            <p className="text-sm text-gray-400">Sets the runtime optimization mode. Modes: <code>'speed'</code>, <code>'size'</code>, <code>'auto'</code>.</p>
          </div>
          <div className="glass p-6 rounded-xl">
            <h3 className="text-brand-yellow font-mono mb-2">Beejs.version</h3>
            <p className="text-sm text-gray-400">Returns the current Beejs runtime version.</p>
          </div>
          <div className="glass p-6 rounded-xl">
            <h3 className="text-brand-yellow font-mono mb-2">Beejs.env</h3>
            <p className="text-sm text-gray-400">Access to system environment variables with optimized lookup.</p>
          </div>
        </div>
      </section>

      <div className="mt-20 pt-8 border-t border-white/10 flex justify-between">
        <Link to="/docs/cli-usage" className="text-gray-500 hover:text-white">← CLI Usage</Link>
        <Link to="/docs/modules" className="text-brand-yellow hover:underline flex items-center space-x-2">
          <span>Modules</span>
          <ChevronRight className="w-4 h-4 ml-1" />
        </Link>
      </div>
    </div>
  )
}

function ModulesContent() {
  return (
    <div className="space-y-8">
      <h1 className="text-4xl font-bold mb-4 text-white">Modules</h1>
      <p className="text-gray-400">Standard and native modules built into the runtime core.</p>

      <section className="space-y-6">
        <h2 className="text-2xl font-bold border-b border-white/10 pb-2 text-white">Standard Support</h2>
        <p>Beejs supports both ESM (ECMAScript Modules) and CommonJS by default.</p>
      </section>

      <section className="space-y-6">
        <h2 className="text-2xl font-bold border-b border-white/10 pb-2 text-white">Native Rust Modules</h2>
        <p>For maximum performance, several modules are implemented directly in Rust:</p>
        <ul className="list-disc list-inside space-y-2 text-gray-400 font-mono text-sm ml-4">
          <li><code>beejs:fs</code> - File system operations via Tokio.</li>
          <li><code>beejs:net</code> - High-performance networking.</li>
          <li><code>beejs:ai</code> - AI batch processing and model inference.</li>
        </ul>
      </section>

      <div className="mt-20 pt-8 border-t border-white/10 flex justify-between">
        <Link to="/docs/api-reference" className="text-gray-500 hover:text-white">← API Reference</Link>
        <Link to="/blog" className="text-brand-yellow hover:underline flex items-center space-x-2">
          <span>Read the Blog</span>
          <ChevronRight className="w-4 h-4 ml-1" />
        </Link>
      </div>
    </div>
  )
}
