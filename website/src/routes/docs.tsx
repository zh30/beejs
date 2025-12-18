import { Link } from 'react-router-dom'
import { Book, ChevronRight, Code, Terminal, Layers, Cpu, Zap } from 'lucide-react'

export default function DocsComponent() {
  const sections = [
    {
      title: 'Getting Started',
      items: [
        { name: 'Introduction', icon: <Book className="w-4 h-4" /> },
        { name: 'Installation', icon: <Terminal className="w-4 h-4" /> },
        { name: 'Quick Start', icon: <Zap className="w-4 h-4" /> },
      ]
    },
    {
      title: 'Core Concepts',
      items: [
        { name: 'V8 Isolate Pool', icon: <Cpu className="w-4 h-4" /> },
        { name: 'JIT Optimization', icon: <Zap className="w-4 h-4" /> },
        { name: 'Memory Management', icon: <Layers className="w-4 h-4" /> },
      ]
    },
    {
      title: 'Reference',
      items: [
        { name: 'CLI Usage', icon: <Code className="w-4 h-4" /> },
        { name: 'API Reference', icon: <Book className="w-4 h-4" /> },
        { name: 'Modules', icon: <Layers className="w-4 h-4" /> },
      ]
    }
  ]

  return (
    <div className="max-w-7xl mx-auto flex flex-col md:flex-row py-12 px-4 md:px-8 gap-12">
      {/* Sidebar */}
      <aside className="w-full md:w-64 shrink-0">
        <nav className="sticky top-24 space-y-8">
          {sections.map((section) => (
            <div key={section.title}>
              <h3 className="text-xs font-bold uppercase tracking-widest text-gray-500 mb-4 px-2">
                {section.title}
              </h3>
              <ul className="space-y-1">
                {section.items.map((item) => (
                  <li key={item.name}>
                    <button className="w-full flex items-center space-x-3 px-3 py-2 rounded-lg text-sm transition-colors text-gray-400 hover:text-white hover:bg-white/5 group text-left">
                      <span className="text-gray-600 group-hover:text-brand-yellow transition-colors">
                        {item.icon}
                      </span>
                      <span>{item.name}</span>
                    </button>
                  </li>
                ))}
              </ul>
            </div>
          ))}
        </nav>
      </aside>

      {/* Content */}
      <main className="grow max-w-3xl prose prose-invert prose-brand text-gray-300">
        <div className="mb-12">
          <h1 className="text-4xl font-bold mb-4 text-white">Introduction to Beejs</h1>
          <p className="text-xl text-gray-400 leading-relaxed">
            Beejs is a high-performance JavaScript/TypeScript runtime built with Rust and V8.
            It's designed to be the fastest engine for executing server-side scripts,
            optimized for the AI era.
          </p>
        </div>

        <section className="mb-12 space-y-6">
          <h2 className="text-2xl font-bold border-b border-white/10 pb-2 text-white">Why Beejs?</h2>
          <p className="text-gray-400">
            Current runtimes like Node.js and even Bun have overhead that can be minimized.
            Beejs takes a unique approach to runtime management by:
          </p>
          <ul className="list-disc list-inside space-y-3 text-gray-400 ml-4">
            <li><strong className="text-white">Aggressive Isolate Pooling</strong>: Reusing V8 isolates to achieve an 11ms startup time.</li>
            <li><strong className="text-white">Smart Memory Pools</strong>: Pre-allocating memory for heavy workloads.</li>
            <li><strong className="text-white">Rust-First I/O</strong>: Using Tokio for non-blocking, Zero-Copy data transfer.</li>
          </ul>
        </section>

        <section className="mb-12 space-y-6">
          <h2 className="text-2xl font-bold border-b border-white/10 pb-2 text-white">Basic Installation</h2>
          <div className="glass rounded-xl p-4 bg-black/40 font-mono text-sm group">
            <div className="flex justify-between items-center text-gray-500 mb-2">
              <span>Terminal</span>
              <button className="hover:text-white transition-colors">Copy</button>
            </div>
            <code className="text-brand-yellow">
              $ curl -fsSL https://beejs.dev/install.sh | sh
            </code>
          </div>
          <p className="text-gray-400">
            Alternatively, you can download the latest binary from our GitHub Releases page.
          </p>
        </section>

        <section className="mb-12 space-y-6">
          <h2 className="text-2xl font-bold border-b border-white/10 pb-2 text-white">Your First Script</h2>
          <p className="text-gray-400">Create a file named <code className="text-white">hello.ts</code>:</p>
          <div className="glass rounded-xl p-4 bg-black/40 font-mono text-sm leading-relaxed">
            <pre>
              <code className="text-gray-300">
                {`const message: string = "Hello Beejs!";
console.log(message);

// Use built-in performance optimization
Beejs.optimize('speed');`}
              </code>
            </pre>
          </div>
          <p className="text-gray-400">Run it with:</p>
          <div className="glass rounded-xl p-4 bg-black/40 font-mono text-sm text-brand-yellow">
            $ beejs hello.ts
          </div>
        </section>

        <div className="mt-20 pt-8 border-t border-white/10 flex justify-between items-center text-sm">
          <Link to="/" className="text-gray-500 hover:text-white flex items-center space-x-2">
            <span>← Home</span>
          </Link>
          <button className="text-brand-yellow hover:underline flex items-center space-x-2">
            <span>Core Concepts</span>
            <ChevronRight className="w-4 h-4 ml-1" />
          </button>
        </div>
      </main>
    </div>
  )
}
