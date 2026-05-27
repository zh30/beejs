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
    <div className="relative min-h-screen bg-hud-void">
      <div className="absolute inset-0 hud-grid opacity-30 pointer-events-none" />
      <div className="scanline" />

      <div className="max-w-7xl mx-auto px-4 md:px-8 py-20 relative z-10">
        <div className="grid grid-cols-1 lg:grid-cols-[260px_1fr] gap-10">
          <aside className="hud-panel-soft p-6 h-fit sticky top-28">
            <Link
              to="/"
              className="inline-flex items-center text-[10px] uppercase tracking-[0.4em] text-hud-muted hover:text-hud-text transition-colors"
            >
              <ArrowLeft className="w-3 h-3 mr-2" /> {manual.backToHome}
            </Link>
            <div className="mt-8">
              {manual.groups.map((group) => (
                <div key={group.title} className="mb-8">
                  <h4 className="text-[10px] uppercase tracking-[0.4em] text-hud-muted mb-4">
                    {group.title}
                  </h4>
                  <div className="space-y-2">
                    {group.items.map((item) => (
                      <Link
                        key={item.id}
                        to={`/docs/${item.id}`}
                        className={`flex items-center justify-between px-3 py-2 border border-transparent text-xs uppercase tracking-[0.3em] transition-all ${
                          section === item.id
                            ? 'text-hud-text border-hud-accent/50 bg-hud-panel/80'
                            : 'text-hud-muted hover:text-hud-text hover:border-hud-line/80'
                        }`}
                      >
                        <span className="flex items-center gap-2">
                          <span className="text-hud-accent/80">{iconMap[item.id]}</span>
                          {item.label}
                        </span>
                      </Link>
                    ))}
                  </div>
                </div>
              ))}
            </div>
          </aside>

          <motion.main
            key={section}
            initial={{ opacity: 0, y: 16 }}
            animate={{ opacity: 1, y: 0 }}
            transition={{ duration: 0.4 }}
            className="hud-panel p-8 md:p-12"
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
  cards,
  kicker,
}: {
  title: string
  subtitle?: string
  body?: readonly string[]
  list?: readonly string[]
  code?: readonly string[]
  cards?: readonly { readonly title: string; readonly desc: string }[]
  kicker: string
}) {
  return (
    <div>
      <header className="mb-10">
        <p className="hud-tag">{kicker}</p>
        <h1 className="text-3xl md:text-4xl font-display uppercase tracking-[0.18em] mt-4">
          {title}
        </h1>
        {subtitle ? <p className="text-hud-muted mt-3">{subtitle}</p> : null}
      </header>

      {body?.map((paragraph, index) => (
        <p key={`${paragraph}-${index}`} className="text-sm md:text-base text-hud-muted leading-relaxed mb-6">
          {paragraph}
        </p>
      ))}

      {cards && (
        <div className="grid grid-cols-1 md:grid-cols-2 gap-4 my-8">
          {cards.map((card) => (
            <div key={card.title} className="hud-panel-soft p-6">
              <h3 className="text-lg font-display uppercase tracking-[0.18em]">
                {card.title}
              </h3>
              <p className="text-sm text-hud-muted mt-3">{card.desc}</p>
            </div>
          ))}
        </div>
      )}

      {list && (
        <ul className="space-y-3 my-6">
          {list.map((item, index) => (
            <li key={`${item}-${index}`} className="flex items-start gap-3 text-sm text-hud-muted">
              <span className="text-hud-accent">—</span>
              <span>{item}</span>
            </li>
          ))}
        </ul>
      )}

      {code && (
        <div className="space-y-3 mt-6">
          {code.map((snippet, index) => (
            <pre
              key={`${snippet}-${index}`}
              className="hud-panel-soft p-4 text-xs text-hud-text font-mono overflow-x-auto"
            >
              <code>{snippet}</code>
            </pre>
          ))}
        </div>
      )}
    </div>
  )
}
