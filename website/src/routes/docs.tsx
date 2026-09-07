import { Link, useParams } from 'react-router-dom'
import { motion } from 'framer-motion'
import type { ReactNode } from 'react'
import {
  Activity,
  Book,
  Code,
  Cpu,
  Layers,
  Terminal,
  Zap,
  Server,
  ArrowLeft,
} from 'lucide-react'
import { useLang } from '../lib/i18n'

const iconMap: Record<string, ReactNode> = {
  introduction: <Book className="w-4 h-4" />,
  installation: <Terminal className="w-4 h-4" />,
  'quick-start': <Zap className="w-4 h-4" />,
  'v8-isolate-pool': <Cpu className="w-4 h-4" />,
  'jit-optimization': <Activity className="w-4 h-4" />,
  'memory-management': <Layers className="w-4 h-4" />,
  'server-mode': <Server className="w-4 h-4" />,
  'cli-usage': <Code className="w-4 h-4" />,
  'api-reference': <Book className="w-4 h-4" />,
  modules: <Layers className="w-4 h-4" />,
}

export default function DocsComponent() {
  const { section = 'introduction' } = useParams()
  const { copy } = useLang()
  const manual = copy.docs

  const content =
    manual.sections[section as keyof typeof manual.sections] || manual.sections.introduction

  return (
    <div className="relative min-h-screen pt-10 pb-24">
      <div className="max-w-6xl mx-auto px-6 relative z-10">
        <div className="grid grid-cols-1 lg:grid-cols-[260px_1fr] gap-10">
          {/* Sidebar */}
          <aside className="glass-panel rounded-2xl p-6 h-fit sticky top-24 border-zinc-200/80 dark:border-zinc-800">
            <Link
              to="/"
              className="inline-flex items-center text-xs font-mono text-zinc-600 dark:text-zinc-400 hover:text-amber-600 dark:hover:text-amber-400 transition-colors mb-6"
            >
              <ArrowLeft className="w-3.5 h-3.5 mr-2" /> {manual.backToHome}
            </Link>
            <div>
              {manual.groups.map((group) => (
                <div key={group.title} className="mb-6">
                  <h4 className="text-[11px] font-mono uppercase tracking-wider text-zinc-600 dark:text-zinc-400 mb-3 font-semibold">
                    {group.title}
                  </h4>
                  <div className="space-y-1">
                    {group.items.map((item) => (
                      <Link
                        key={item.id}
                        to={`/docs/${item.id}`}
                        className={`flex items-center gap-2.5 px-3 py-2 rounded-xl text-xs font-medium transition-all ${
                          section === item.id
                            ? 'text-amber-700 dark:text-white bg-amber-500/10 border border-amber-500/30 shadow-sm font-semibold'
                            : 'text-zinc-700 dark:text-zinc-400 hover:text-zinc-950 dark:hover:text-zinc-200 hover:bg-zinc-200/60 dark:hover:bg-zinc-800/40'
                        }`}
                      >
                        <span className={section === item.id ? 'text-amber-600 dark:text-amber-400' : 'text-zinc-500 dark:text-zinc-500'}>
                          {iconMap[item.id]}
                        </span>
                        <span>{item.label}</span>
                      </Link>
                    ))}
                  </div>
                </div>
              ))}
            </div>
          </aside>

          {/* Main Doc Content */}
          <motion.main
            key={section}
            initial={{ opacity: 0, y: 16 }}
            animate={{ opacity: 1, y: 0 }}
            transition={{ duration: 0.3 }}
            className="glass-panel rounded-2xl p-8 md:p-12 border-zinc-200/80 dark:border-zinc-800"
          >
            <ManualSection {...content} kicker={manual.title} />
          </motion.main>
        </div>
      </div>
    </div>
  )
}

function ManualSection({
  title,
  subtitle,
  body,
  list,
  code,
  kicker,
}: {
  title: string
  subtitle: string
  body: string
  list?: string[]
  code?: string | string[]
  kicker: string
}) {
  return (
    <div className="space-y-6">
      <div className="flex items-center gap-2 text-xs font-mono text-amber-700 dark:text-amber-400">
        <span>{kicker}</span>
        <span className="text-zinc-400 dark:text-zinc-600">/</span>
        <span className="text-zinc-900 dark:text-zinc-200 font-semibold">{title}</span>
      </div>

      <h1 className="text-3xl md:text-4xl font-extrabold text-zinc-950 dark:text-white font-display tracking-tight">
        {title}
      </h1>

      <p className="text-base text-zinc-700 dark:text-zinc-300 leading-relaxed font-normal">{subtitle}</p>

      <div className="h-px w-full bg-zinc-200/80 dark:bg-zinc-800/80 my-6" />

      <p className="text-sm text-zinc-700 dark:text-zinc-300 leading-relaxed">{body}</p>

      {list && (
        <ul className="space-y-3 my-6">
          {list.map((item) => (
            <li key={item} className="flex items-start gap-3 text-sm text-zinc-800 dark:text-zinc-200">
              <span className="w-1.5 h-1.5 rounded-full bg-amber-500 dark:bg-amber-400 mt-2 shrink-0" />
              <span>{item}</span>
            </li>
          ))}
        </ul>
      )}

      {code && (
        <div className="rounded-xl overflow-hidden bg-[#0a0b0e] border border-zinc-800/80 p-5 font-mono text-xs text-zinc-100 overflow-x-auto leading-relaxed shadow-inner select-text">
          <pre>{Array.isArray(code) ? code.join('\n') : code}</pre>
        </div>
      )}
    </div>
  )
}
