import { motion } from 'framer-motion'
import { useState } from 'react'
import {
  Activity,
  ArrowRight,
  Check,
  CheckCircle2,
  Copy,
  Cpu,
  Download,
  FileCode2,
  Gauge,
  Layers,
  Lock,
  Package,
  Terminal,
  Zap,
} from 'lucide-react'
import { Link } from 'react-router-dom'
import { useLang } from '../lib/i18n'

export default function HomeComponent() {
  const { copy } = useLang()
  const home = copy.home
  const [copied, setCopied] = useState(false)

  const installCommand = 'curl -fsSL https://bee.zhanghe.dev/install.sh | sh'

  const handleCopy = () => {
    navigator.clipboard.writeText(installCommand)
    setCopied(true)
    setTimeout(() => setCopied(false), 2000)
  }

  return (
    <div className="relative overflow-hidden pt-12 pb-24">
      {/* Hero Section */}
      <section className="max-w-6xl mx-auto px-6 pt-12 pb-20 text-center relative z-10">
        <motion.div
          initial={{ opacity: 0, y: 20 }}
          animate={{ opacity: 1, y: 0 }}
          transition={{ duration: 0.5 }}
          className="inline-flex items-center gap-2 px-3.5 py-1.5 rounded-full glass-card border-amber-500/20 text-xs font-mono text-amber-300 mb-8"
        >
          <Zap className="w-3.5 h-3.5 text-amber-400 fill-amber-400" />
          <span>v0.1.0-repair-sprint Released</span>
          <span className="text-zinc-600">•</span>
          <span className="text-zinc-400">100% Test Suite Verified</span>
        </motion.div>

        <motion.h1
          initial={{ opacity: 0, y: 20 }}
          animate={{ opacity: 1, y: 0 }}
          transition={{ duration: 0.5, delay: 0.1 }}
          className="text-5xl sm:text-6xl md:text-7xl font-extrabold tracking-tight max-w-4xl mx-auto leading-[1.1] font-display"
        >
          The High-Performance{' '}
          <span className="gradient-amber">JS/TS Runtime</span> Built for Speed
        </motion.h1>

        <motion.p
          initial={{ opacity: 0, y: 20 }}
          animate={{ opacity: 1, y: 0 }}
          transition={{ duration: 0.5, delay: 0.2 }}
          className="mt-6 text-lg sm:text-xl text-zinc-400 max-w-2xl mx-auto font-normal leading-relaxed"
        >
          Powered by Rust and V8. Instant TypeScript execution, fail-closed security, and native Node/Web API compatibility.
        </motion.p>

        <motion.div
          initial={{ opacity: 0, y: 20 }}
          animate={{ opacity: 1, y: 0 }}
          transition={{ duration: 0.5, delay: 0.3 }}
          className="mt-10 flex flex-col sm:flex-row items-center justify-center gap-4"
        >
          <Link
            to="/docs"
            className="w-full sm:w-auto px-8 py-3.5 rounded-full bg-amber-500 text-zinc-950 font-semibold text-sm hover:bg-amber-400 transition-all flex items-center justify-center gap-2 shadow-lg shadow-amber-500/20 group"
          >
            <span>{home.ctaPrimary}</span>
            <ArrowRight className="w-4 h-4 transition-transform group-hover:translate-x-1" />
          </Link>
          <Link
            to="/blog"
            className="w-full sm:w-auto px-8 py-3.5 rounded-full glass-card hover:bg-zinc-800/60 text-zinc-300 font-semibold text-sm transition-all flex items-center justify-center gap-2"
          >
            <span>{home.ctaSecondary}</span>
          </Link>
        </motion.div>

        {/* Quick Install Bar */}
        <motion.div
          initial={{ opacity: 0, y: 20 }}
          animate={{ opacity: 1, y: 0 }}
          transition={{ duration: 0.5, delay: 0.4 }}
          className="mt-10 max-w-xl mx-auto"
        >
          <div className="glass-panel rounded-2xl p-2.5 flex items-center justify-between gap-3 text-xs font-mono text-zinc-300 border-zinc-800/80">
            <div className="flex items-center gap-2 px-2 overflow-x-auto truncate">
              <Terminal className="w-4 h-4 text-amber-400 shrink-0" />
              <span className="select-all text-zinc-300">{installCommand}</span>
            </div>
            <button
              onClick={handleCopy}
              className="px-3 py-1.5 rounded-xl bg-zinc-800 hover:bg-zinc-700 text-zinc-200 transition-all shrink-0 flex items-center gap-1.5 text-xs font-sans font-medium"
            >
              {copied ? (
                <>
                  <Check className="w-3.5 h-3.5 text-emerald-400" />
                  <span className="text-emerald-400">Copied</span>
                </>
              ) : (
                <>
                  <Copy className="w-3.5 h-3.5 text-zinc-400" />
                  <span>Copy</span>
                </>
              )}
            </button>
          </div>
        </motion.div>
      </section>

      {/* Terminal Code Sandbox Preview */}
      <section className="max-w-4xl mx-auto px-6 pb-24 relative z-10">
        <motion.div
          initial={{ opacity: 0, scale: 0.96 }}
          whileInView={{ opacity: 1, scale: 1 }}
          viewport={{ once: true }}
          transition={{ duration: 0.6 }}
          className="glass-panel rounded-2xl overflow-hidden shadow-2xl border-zinc-800"
        >
          <div className="px-4 py-3 bg-zinc-900/90 border-b border-zinc-800/80 flex items-center justify-between">
            <div className="flex items-center gap-2">
              <div className="w-3 h-3 rounded-full bg-red-500/80" />
              <div className="w-3 h-3 rounded-full bg-amber-500/80" />
              <div className="w-3 h-3 rounded-full bg-emerald-500/80" />
              <span className="ml-2 text-xs font-mono text-zinc-400">app.ts — Beejs Runtime</span>
            </div>
            <span className="text-[10px] font-mono px-2 py-0.5 rounded bg-zinc-800 text-zinc-400">
              TypeScript Native
            </span>
          </div>
          <div className="p-6 font-mono text-sm leading-relaxed overflow-x-auto text-zinc-300 bg-[#0a0b0e]">
            <div className="text-zinc-500">// Instant TypeScript execution with V8 engine</div>
            <div>
              <span className="text-purple-400">import</span> &#123; createServer &#125;{' '}
              <span className="text-purple-400">from</span> <span className="text-emerald-400">'node:http'</span>;
            </div>
            <br />
            <div>
              <span className="text-blue-400">const</span> <span className="text-amber-300">server</span> ={' '}
              <span className="text-blue-400">createServer</span>((req, res) =&gt; &#123;
            </div>
            <div className="pl-4">
              res.<span className="text-blue-400">writeHead</span>(<span className="text-orange-400">200</span>, &#123;{' '}
              <span className="text-emerald-400">'Content-Type'</span>: <span className="text-emerald-400">'text/plain'</span> &#125;);
            </div>
            <div className="pl-4">
              res.<span className="text-blue-400">end</span>(<span className="text-emerald-400">'Hello from Beejs Runtime!'</span>);
            </div>
            <div>&#125;);</div>
            <br />
            <div>
              server.<span className="text-blue-400">listen</span>(<span className="text-orange-400">3000</span>, () =&gt; &#123;
            </div>
            <div className="pl-4 text-emerald-400">
              console.<span className="text-blue-400">log</span>(<span className="text-emerald-400">'🚀 Server running at http://localhost:3000'</span>);
            </div>
            <div>&#125;);</div>
            <div className="mt-6 pt-4 border-t border-zinc-800/60 text-xs text-emerald-400 flex items-center gap-2">
              <CheckCircle2 className="w-4 h-4 text-emerald-400" />
              <span>$ bee run app.ts</span>
              <span className="text-zinc-500 ml-auto">Execution time: 4ms</span>
            </div>
          </div>
        </motion.div>
      </section>

      {/* Metrics & Performance Grid */}
      <section className="max-w-6xl mx-auto px-6 pb-24 relative z-10">
        <div className="grid grid-cols-2 md:grid-cols-4 gap-4">
          {home.telemetry.map((item) => (
            <div key={item.label} className="glass-card rounded-2xl p-5 border-zinc-800/70">
              <div className="text-xs text-zinc-500 uppercase tracking-wider font-mono">{item.label}</div>
              <div className="text-2xl font-bold text-white mt-2 font-display">{item.value}</div>
              <div className="text-xs text-amber-400 font-mono mt-1">{item.delta}</div>
            </div>
          ))}
        </div>
      </section>

      {/* Core Capabilities */}
      <section className="max-w-6xl mx-auto px-6 pb-24 relative z-10">
        <div className="text-center max-w-2xl mx-auto mb-14">
          <h2 className="text-3xl font-bold text-white font-display">Engine Architecture</h2>
          <p className="mt-3 text-zinc-400 text-sm">
            Everything you need for ultra-fast, secure, and modern JavaScript/TypeScript applications.
          </p>
        </div>

        <div className="grid grid-cols-1 md:grid-cols-3 gap-6">
          <FeatureCard
            icon={<Cpu className="w-6 h-6 text-amber-400" />}
            title="V8 JIT Engine"
            desc="Built on Google V8 for maximum JS execution performance and memory efficiency."
          />
          <FeatureCard
            icon={<FileCode2 className="w-6 h-6 text-amber-400" />}
            title="TypeScript Native"
            desc="Zero-config instant TS & TSX transpilation with Source Map error diagnostic alignment."
          />
          <FeatureCard
            icon={<Lock className="w-6 h-6 text-amber-400" />}
            title="Fail-Closed Security"
            desc="Granular permission broker for FS, network, process, and environment isolation."
          />
          <FeatureCard
            icon={<Layers className="w-6 h-6 text-amber-400" />}
            title="Node.js & Web APIs"
            desc="Supported fs.promises, http, crypto, fetch, Blob, Streams, and WebCrypto standards."
          />
          <FeatureCard
            icon={<Gauge className="w-6 h-6 text-amber-400" />}
            title="Sub-Millisecond Boot"
            desc="Optimized Tokio event loop and V8 isolate bootstrap for near-instant cold starts."
          />
          <FeatureCard
            icon={<Package className="w-6 h-6 text-amber-400" />}
            title="Built-in Test Runner"
            desc="Jest-style assertion suite, matcher tracking, and parallel test runner."
          />
        </div>
      </section>
    </div>
  )
}

function FeatureCard({ icon, title, desc }: { icon: React.ReactNode; title: string; desc: string }) {
  return (
    <div className="glass-card rounded-2xl p-6 border-zinc-800/80 group">
      <div className="w-12 h-12 rounded-xl bg-amber-500/10 border border-amber-500/20 flex items-center justify-center mb-5 group-hover:scale-110 transition-transform">
        {icon}
      </div>
      <h3 className="text-lg font-semibold text-white font-display">{title}</h3>
      <p className="mt-2 text-sm text-zinc-400 leading-relaxed font-normal">{desc}</p>
    </div>
  )
}
