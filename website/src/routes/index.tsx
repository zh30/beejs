import { motion } from 'framer-motion'
import { Rocket, Zap, Shield, Brain, Cpu, Code, Terminal } from 'lucide-react'
import { Link } from 'react-router-dom'

export default function HomeComponent() {
  return (
    <div className="flex flex-col overflow-hidden">
      {/* Hero Section */}
      <section className="relative h-[80vh] flex items-center justify-center px-4">
        <div className="absolute inset-0 bg-[radial-gradient(circle_at_center,_var(--tw-gradient-stops))] from-brand-yellow/10 via-transparent to-transparent opacity-50" />
        <div className="relative text-center max-w-4xl">
          <motion.div
            initial={{ opacity: 0, y: 20 }}
            animate={{ opacity: 1, y: 0 }}
            transition={{ duration: 0.6 }}
          >
            <h1 className="text-5xl md:text-8xl font-black mb-6 tracking-tighter leading-none text-white">
              SPEED OF <span className="text-brand-yellow">LIGHT</span> <br />
              JS RUNTIME
            </h1>
            <p className="text-lg md:text-2xl text-gray-400 mb-10 max-w-2xl mx-auto leading-relaxed">
              Beejs: The ultra-high-performance JavaScript/TypeScript runtime for the AI age.
              Built with Rust + V8. 11ms startup.
            </p>
            <div className="flex flex-col md:flex-row items-center justify-center space-y-4 md:space-y-0 md:space-x-4">
              <Link to="/docs" className="w-full md:w-auto px-8 py-4 bg-brand-yellow text-brand-black font-bold rounded-xl text-lg hover:scale-105 transition-transform text-center">
                Get Started
              </Link>
              <Link to="/docs" className="w-full md:w-auto px-8 py-4 glass text-white font-bold rounded-xl text-lg hover:bg-white/10 transition-colors text-center">
                View Documentation
              </Link>
            </div>
          </motion.div>
        </div>
      </section>

      {/* Benchmarks Section */}
      <section className="py-24 px-4 bg-brand-gray/30">
        <div className="max-w-6xl mx-auto">
          <div className="text-center mb-16">
            <h2 className="text-3xl md:text-5xl font-bold mb-4 text-white">Benchmarks</h2>
            <p className="text-gray-400">Beejs crushes the competition in every metric.</p>
          </div>

          <div className="grid grid-cols-1 md:grid-cols-3 gap-8">
            <BenchmarkCard
              label="Startup Time"
              beeValue="11ms"
              bunValue="72ms"
              improvement="+84.7%"
              icon={<Rocket className="w-6 h-6" />}
            />
            <BenchmarkCard
              label="Memory Usage"
              beeValue="82MB"
              bunValue="102MB"
              improvement="-19.6%"
              icon={<Cpu className="w-6 h-6" />}
            />
            <BenchmarkCard
              label="Concurrency"
              beeValue="11,200"
              bunValue="8,200"
              improvement="+36.6%"
              icon={<Zap className="w-6 h-6" />}
            />
          </div>
        </div>
      </section>

      {/* Features Section */}
      <section className="py-24 px-4">
        <div className="max-w-6xl mx-auto">
          <div className="text-center mb-16">
            <h2 className="text-3xl md:text-5xl font-bold mb-4 text-white">Why Beejs?</h2>
            <p className="text-gray-400">Engineered for the most demanding applications.</p>
          </div>

          <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-8">
            <FeatureCard
              title="V8 Isolate Pooling"
              desc="86% faster startup through intelligent V8 instance reuse."
              icon={<Shield className="w-6 h-6" />}
            />
            <FeatureCard
              title="AI Optimized"
              desc="Built-in AI batch processor and memory pre-allocation."
              icon={<Brain className="w-6 h-6" />}
            />
            <FeatureCard
              title="Smart JIT"
              desc="Dynamic threshold adjustment based on execution frequency."
              icon={<Zap className="w-6 h-6" />}
            />
            <FeatureCard
              title="Zero-Copy I/O"
              desc="Efficient data transfer between Rust core and V8 heap."
              icon={<Rocket className="w-6 h-6" />}
            />
            <FeatureCard
              title="TypeScript Native"
              desc="First-class TS support with zero-config compilation."
              icon={<Code className="w-6 h-6" />}
            />
            <FeatureCard
              title="Modern Package Manager"
              desc="NPM/Yarn compatible. Lightning fast dependency resolution."
              icon={<Cpu className="w-6 h-6" />}
            />
          </div>
        </div>
      </section>

      {/* Code Sample Section */}
      <section className="py-24 px-4">
        <div className="max-w-4xl mx-auto">
          <div className="glass rounded-2xl p-8 md:p-12 border border-white/10 shadow-2xl bg-brand-black/40">
            <div className="flex items-center space-x-2 mb-6">
              <div className="w-3 h-3 rounded-full bg-red-500" />
              <div className="w-3 h-3 rounded-full bg-yellow-500" />
              <div className="w-3 h-3 rounded-full bg-green-500" />
              <span className="text-xs text-gray-500 ml-2 font-mono uppercase tracking-wider">example.ts</span>
            </div>
            <pre className="text-sm md:text-base font-mono text-gray-300 overflow-x-auto leading-relaxed">
              <code>{`// Beejs AI 推理示例
const model = await Beejs.AI.load('llama-3');

const response = await model.generate({
  prompt: "Why is Beejs so fast?",
  stream: true
});

console.log(response);

// 运行: beejs --optimize speed example.js`}</code>
            </pre>
          </div>
        </div>
      </section>
    </div>
  )
}

function BenchmarkCard({ label, beeValue, bunValue, improvement, icon }: any) {
  return (
    <div className="glass p-8 rounded-2xl border border-white/10 relative overflow-hidden group hover:border-brand-yellow/30 transition-colors">
      <div className="mb-6 text-brand-yellow">{icon}</div>
      <h3 className="text-gray-400 text-sm font-medium uppercase tracking-wider mb-2">{label}</h3>
      <div className="flex items-baseline space-x-4 mb-4">
        <span className="text-4xl font-bold text-white tracking-tighter">{beeValue}</span>
        <span className="text-gray-500 line-through text-lg">{bunValue}</span>
      </div>
      <div className="inline-block px-3 py-1 bg-green-500/20 text-green-400 text-[10px] font-black rounded-full uppercase tracking-widest">
        {improvement} IMPROVEMENT
      </div>
    </div>
  )
}

function FeatureCard({ title, desc, icon }: any) {
  return (
    <div className="p-8 rounded-2xl bg-white/5 border border-white/5 hover:bg-white/10 hover:border-white/10 transition-all group">
      <div className="w-12 h-12 rounded-xl bg-brand-yellow/10 flex items-center justify-center mb-6 group-hover:scale-110 transition-transform text-brand-yellow">
        {icon}
      </div>
      <h3 className="text-xl font-bold mb-3 text-white">{title}</h3>
      <p className="text-gray-400 text-sm leading-relaxed">{desc}</p>
    </div>
  )
}
