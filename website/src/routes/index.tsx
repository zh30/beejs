import { motion } from 'framer-motion'
import { Rocket, Zap, Shield, Brain, Cpu, Code, Activity, Server } from 'lucide-react'
import { Link } from 'react-router-dom'
import { BeeLogo } from '../components/Logo'

export default function HomeComponent() {
  return (
    <div className="flex flex-col overflow-hidden bg-brand-black relative">
      {/* Background Decor */}
      <div className="absolute inset-0 cyber-grid opacity-30 pointer-events-none" />
      <div className="scanline pointer-events-none" />

      {/* Hero Section */}
      <section className="relative min-h-[90vh] flex items-center justify-center px-4 overflow-hidden pt-20">
        <div className="absolute top-1/2 left-1/2 -translate-x-1/2 -translate-y-1/2 w-150 h-150 bg-brand-yellow/5 rounded-full blur-[120px] pointer-events-none" />

        <div className="relative text-center max-w-6xl z-10 w-full">
          <motion.div
            initial={{ opacity: 0, scale: 0.8 }}
            animate={{ opacity: 1, scale: 1 }}
            transition={{ duration: 0.8, ease: "easeOut" }}
            className="mb-8 flex justify-center"
          >
            {/* Cyber Bee Graphic */}
            <div className="relative w-24 h-24 md:w-40 md:h-40">
              <div className="absolute inset-0 bg-brand-yellow/20 rounded-full animate-pulse blur-xl" />
              <BeeLogo className="w-full h-full" />
            </div>
          </motion.div>

          <motion.div
            initial={{ opacity: 0, y: 30 }}
            animate={{ opacity: 1, y: 0 }}
            transition={{ duration: 0.8, delay: 0.2 }}
          >
            <h1 className="text-5xl md:text-9xl font-black mb-6 tracking-tighter leading-[0.9] text-white uppercase italic">
              Ultra <span className="text-brand-yellow glow-text">Fast</span> <br />
              <span className="text-gray-500">AI Runtime</span>
            </h1>
            <p className="text-base md:text-xl text-gray-400 mb-10 max-w-2xl mx-auto leading-relaxed font-mono uppercase tracking-widest px-4">
              Building the backbone of the next billion AI agents. <br className="hidden md:block" />
              11ms startup. Zero compromise.
            </p>
            <div className="flex flex-col md:flex-row items-center justify-center space-y-4 md:space-y-0 md:space-x-4 px-4">
              <Link to="/docs" className="w-full md:w-auto px-10 py-5 bg-brand-yellow text-brand-black font-black rounded-none -skew-x-12 text-lg hover:bg-white transition-all text-center uppercase">
                <span className="inline-block skew-x-12">Initialize Docs</span>
              </Link>
              <Link to="/blog" className="w-full md:w-auto px-10 py-5 glass text-white font-black rounded-none -skew-x-12 text-lg hover:bg-white/10 transition-all text-center uppercase border-brand-yellow/20">
                <span className="inline-block skew-x-12">Read Terminal</span>
              </Link>
            </div>
          </motion.div>
        </div>

        {/* Floating Decors */}
        <div className="absolute bottom-10 left-10 hidden lg:block text-[10px] font-mono text-gray-600 uppercase tracking-[0.5em] vertical-rl">
          System Status: Operational // Core: V8-Rust // Latency: 11ms
        </div>
      </section>

      {/* Benchmarks Section */}
      <section className="py-24 px-4 relative">
        <div className="max-w-6xl mx-auto">
          <div className="flex flex-col md:flex-row md:items-end justify-between mb-20 gap-8">
            <div className="max-w-xl">
              <h2 className="text-xs font-black text-brand-yellow uppercase tracking-[0.3em] mb-4">Telemetry.Data</h2>
              <h3 className="text-4xl md:text-6xl font-black text-white leading-tight uppercase">
                Performance <br />
                <span className="text-gray-500">Over-Engineered.</span>
              </h3>
            </div>
            <div className="text-gray-500 font-mono text-sm max-w-sm mb-2">
              Our benchmarks are verified across 10,000+ simulation cycles. Beejs isn't just faster—it's mathematically superior.
            </div>
          </div>

          <div className="grid grid-cols-1 md:grid-cols-3 gap-6">
            <BenchmarkModule
              label="Sync Startup"
              value="11ms"
              comparison="v.s. 72ms"
              percent="+84.7%"
              icon={<Activity className="w-5 h-5" />}
              color="text-brand-yellow"
            />
            <BenchmarkModule
              label="Isolate Heap"
              value="82MB"
              comparison="v.s. 102MB"
              percent="-19.6%"
              icon={<Server className="w-5 h-5" />}
              color="text-blue-400"
            />
            <BenchmarkModule
              label="Concurrency"
              value="11.2K"
              comparison="v.s. 8.2K"
              percent="+36.6%"
              icon={<Zap className="w-5 h-5" />}
              color="text-purple-400"
            />
          </div>
        </div>
      </section>

      {/* Features Grid */}
      <section className="py-24 px-4 bg-brand-gray/20 border-y border-white/5 relative">
        <div className="max-w-6xl mx-auto">
          <div className="text-center mb-20">
            <h2 className="text-4xl md:text-7xl font-black text-white uppercase italic tracking-tighter">
              The <span className="text-brand-yellow">Hive</span> Core
            </h2>
          </div>

          <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-1">
            <FeatureCard
              icon={<Shield />}
              title="V8 Isolate Pool"
              desc="Instant-on isolates eliminate cold-start penalties globally."
            />
            <FeatureCard
              icon={<Brain />}
              title="AI Co-Processor"
              desc="Built-in hardware acceleration for LLM and Vision tasks."
            />
            <FeatureCard
              icon={<Zap />}
              title="Turbo JIT"
              desc="Dynamic thresholding shifts into high gear based on load."
            />
            <FeatureCard
              icon={<Rocket />}
              title="Zero-Copy I/O"
              desc="No serialization bottle-necks between Rust and JS heap."
            />
            <FeatureCard
              icon={<Code />}
              title="TS Fundamental"
              desc="TypeScript is not a second class citizen. It is the language."
            />
            <FeatureCard
              icon={<Activity />}
              title="Core Observer"
              desc="Real-time telemetry and self-healing runtime state."
            />
          </div>
        </div>
      </section>

      {/* Final CTA */}
      <section className="py-32 px-4 flex flex-col items-center justify-center text-center">
        <motion.div
          initial={{ opacity: 0 }}
          whileInView={{ opacity: 1 }}
          className="max-w-4xl"
        >
          <h2 className="text-5xl md:text-8xl font-black text-white mb-12 uppercase tracking-tighter italic">
            Ready to <span className="text-brand-yellow">Ascend?</span>
          </h2>
          <Link to="/docs" className="inline-block px-12 py-6 bg-white text-brand-black font-black text-xl hover:bg-brand-yellow transition-all uppercase -skew-x-12">
            <span className="inline-block skew-x-12">Get Started Now</span>
          </Link>
          <p className="mt-8 text-gray-600 font-mono text-xs uppercase tracking-widest">
            v0.1.0-STABLE // OPEN SOURCE // RUST CORE
          </p>
        </motion.div>
      </section>
    </div>
  )
}

function BenchmarkModule({ label, value, comparison, percent, icon, color }: any) {
  return (
    <div className="glass p-8 neon-border-animated relative group flex flex-col justify-between h-full">
      <div className="flex items-center justify-between mb-12">
        <div className={`${color} opacity-80`}>{icon}</div>
        <div className="text-[10px] font-bold uppercase tracking-widest text-gray-500">{label}</div>
      </div>
      <div>
        <div className="text-5xl font-black text-white mb-4 tracking-tighter">{value}</div>
        <div className="flex items-center justify-between">
          <div className="text-[10px] font-mono text-gray-600">{comparison}</div>
          <div className="text-xs font-black text-green-500">{percent}</div>
        </div>
      </div>
      {/* Decorative dots */}
      <div className="absolute top-2 right-2 flex space-x-1">
        <div className="w-1 h-1 rounded-full bg-white/20" />
        <div className="w-1 h-1 rounded-full bg-white/20" />
      </div>
    </div>
  )
}

function FeatureCard({ icon, title, desc }: any) {
  return (
    <div className="p-10 bg-brand-black/40 hover:bg-white/5 border border-white/5 transition-all group relative overflow-hidden">
      <div className="w-12 h-12 text-brand-yellow mb-8 transition-transform group-hover:scale-110 group-hover:glow-text">
        {icon}
      </div>
      <h3 className="text-2xl font-black text-white mb-4 uppercase italic tracking-tighter">{title}</h3>
      <p className="text-gray-500 text-sm leading-relaxed max-w-xs">{desc}</p>

      {/* Corner decor */}
      <div className="absolute top-0 right-0 w-8 h-8 pointer-events-none">
        <div className="absolute top-2 right-2 w-px h-4 bg-white/10" />
        <div className="absolute top-2 right-2 w-4 h-px bg-white/10" />
      </div>
    </div>
  )
}
