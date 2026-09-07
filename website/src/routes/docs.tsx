import { Link, useParams } from 'react-router-dom'
import { motion } from 'framer-motion'
import { useState, type ReactNode } from 'react'
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
  Sparkles,
  Package,
  Copy,
  Check,
} from 'lucide-react'
import ReactMarkdown from 'react-markdown'
import remarkGfm from 'remark-gfm'
import { useLang } from '../lib/i18n'

const iconMap: Record<string, ReactNode> = {
  introduction: <Book className="w-4 h-4" />,
  installation: <Terminal className="w-4 h-4" />,
  'quick-start': <Zap className="w-4 h-4" />,
  'v8-isolate-pool': <Cpu className="w-4 h-4" />,
  'jit-optimization': <Activity className="w-4 h-4" />,
  'ai-engine': <Sparkles className="w-4 h-4" />,
  'server-mode': <Server className="w-4 h-4" />,
  'memory-management': <Layers className="w-4 h-4" />,
  'cli-usage': <Code className="w-4 h-4" />,
  'api-reference': <Book className="w-4 h-4" />,
  modules: <Package className="w-4 h-4" />,
}

const docModules = import.meta.glob('../docs/*.md', {
  query: '?raw',
  eager: true,
  import: 'default',
}) as Record<string, string>

function parseFrontmatter(raw: string): { data: Record<string, string>; content: string } {
  const match = raw.match(/^---\s*([\s\S]*?)\s*---\s*([\s\S]*)$/)
  if (!match) return { data: {}, content: raw }

  const yaml = match[1]
  const content = match[2]
  const data: Record<string, string> = {}

  yaml
    .split('\n')
    .filter(Boolean)
    .forEach((line) => {
      const [key, ...valueParts] = line.split(':')
      if (key && valueParts.length > 0) {
        data[key.trim()] = valueParts.join(':').trim().replace(/^["']|["']$/g, '')
      }
    })

  return { data, content }
}

function getDocContent(section: string, lang: string) {
  // 1. Try language-specific file (e.g. .zh.md, .es.md, etc.)
  if (lang !== 'en') {
    const langKey = `../docs/${section}.${lang}.md`
    if (docModules[langKey]) return parseFrontmatter(docModules[langKey])
  }

  // 2. If lang is 'zh' or fallback to Chinese
  if (lang === 'zh') {
    const zhKey = `../docs/${section}.zh.md`
    if (docModules[zhKey]) return parseFrontmatter(docModules[zhKey])
  }

  // 3. Try standard English file
  const enKey = `../docs/${section}.md`
  if (docModules[enKey]) return parseFrontmatter(docModules[enKey])

  // 4. Fallback to zh file if en missing
  const fallbackZh = `../docs/${section}.zh.md`
  if (docModules[fallbackZh]) return parseFrontmatter(docModules[fallbackZh])

  return null
}

function PreBlock({ children }: { children?: ReactNode }) {
  const [copied, setCopied] = useState(false)

  const extractText = (node: any): string => {
    if (typeof node === 'string') return node
    if (Array.isArray(node)) return node.map(extractText).join('')
    if (node?.props?.children) return extractText(node.props.children)
    return ''
  }

  const text = extractText(children).trim()

  const handleCopy = () => {
    navigator.clipboard.writeText(text)
    setCopied(true)
    setTimeout(() => setCopied(false), 2000)
  }

  return (
    <div className="relative group/pre my-6 rounded-2xl overflow-hidden border border-zinc-800/80 bg-[#0a0b0e] shadow-xl">
      <div className="flex items-center justify-between px-4 py-2 bg-zinc-900/80 border-b border-zinc-800/80 text-xs font-mono text-zinc-400">
        <span className="flex items-center gap-1.5 text-zinc-400">
          <Terminal className="w-3.5 h-3.5 text-amber-500/80" />
          <span>terminal</span>
        </span>
        <button
          onClick={handleCopy}
          aria-label="Copy code"
          className="inline-flex items-center gap-1 text-[11px] text-zinc-400 hover:text-white px-2 py-0.5 rounded transition-colors bg-zinc-800/50 hover:bg-zinc-700/50"
        >
          {copied ? (
            <>
              <Check className="w-3.5 h-3.5 text-emerald-400" />
              <span className="text-emerald-400 font-medium">Copied</span>
            </>
          ) : (
            <>
              <Copy className="w-3.5 h-3.5" />
              <span>Copy</span>
            </>
          )}
        </button>
      </div>
      <pre className="p-5 overflow-x-auto text-xs font-mono leading-relaxed text-zinc-100 select-text m-0 bg-transparent!">
        {children}
      </pre>
    </div>
  )
}

function BlockquoteBlock({ children }: { children?: ReactNode }) {
  return (
    <blockquote className="my-6 border-l-4 border-amber-500 bg-amber-500/10 dark:bg-amber-500/15 rounded-r-2xl p-4 sm:p-5 text-sm not-italic text-zinc-800 dark:text-zinc-200 shadow-sm leading-relaxed">
      {children}
    </blockquote>
  )
}

function TableBlock({ children }: { children?: ReactNode }) {
  return (
    <div className="my-6 overflow-x-auto rounded-2xl border border-zinc-200/80 dark:border-zinc-800 shadow-sm bg-white/40 dark:bg-zinc-900/20">
      <table className="w-full text-left border-collapse text-xs sm:text-sm m-0">
        {children}
      </table>
    </div>
  )
}

export default function DocsComponent() {
  const { section = 'introduction' } = useParams()
  const { copy, lang } = useLang()
  const manual = copy.docs

  const docData = getDocContent(section, lang)

  // Find active group title
  let currentGroupTitle = manual.title
  for (const group of manual.groups) {
    if (group.items.some((item) => item.id === section)) {
      currentGroupTitle = group.title
      break
    }
  }

  // Fallback content if no markdown found
  const fallbackContent =
    manual.sections[section as keyof typeof manual.sections] || manual.sections.introduction

  const title = docData?.data?.title || fallbackContent?.title || section
  const subtitle = docData?.data?.subtitle || fallbackContent?.subtitle || ''

  return (
    <div className="relative min-h-screen pt-10 pb-24">
      <div className="max-w-6xl mx-auto px-6 relative z-10">
        <div className="grid grid-cols-1 lg:grid-cols-[280px_1fr] gap-10">
          {/* Sidebar */}
          <aside className="glass-panel rounded-2xl p-6 h-fit sticky top-24 border-zinc-200/80 dark:border-zinc-800">
            <Link
              to="/"
              className="inline-flex items-center text-xs font-mono text-zinc-600 dark:text-zinc-400 hover:text-amber-600 dark:hover:text-amber-400 transition-colors mb-6"
            >
              <ArrowLeft className="w-3.5 h-3.5 mr-2" /> {manual.backToHome}
            </Link>
            <div className="space-y-6">
              {manual.groups.map((group) => (
                <div key={group.title}>
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
                        <span
                          className={
                            section === item.id
                              ? 'text-amber-600 dark:text-amber-400'
                              : 'text-zinc-500 dark:text-zinc-500'
                          }
                        >
                          {iconMap[item.id] || <Book className="w-4 h-4" />}
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
            className="glass-panel rounded-2xl p-8 md:p-12 border-zinc-200/80 dark:border-zinc-800 min-w-0"
          >
            {/* Header / Breadcrumb */}
            <div className="space-y-4 mb-8">
              <div className="flex items-center gap-2 text-xs font-mono text-amber-700 dark:text-amber-400">
                <span>{manual.title}</span>
                <span className="text-zinc-400 dark:text-zinc-600">/</span>
                <span>{currentGroupTitle}</span>
                <span className="text-zinc-400 dark:text-zinc-600">/</span>
                <span className="text-zinc-900 dark:text-zinc-200 font-semibold">{title}</span>
              </div>

              <h1 className="text-3xl md:text-4xl font-extrabold text-zinc-950 dark:text-white font-display tracking-tight">
                {title}
              </h1>

              {subtitle && (
                <p className="text-base text-zinc-700 dark:text-zinc-300 leading-relaxed font-normal">
                  {subtitle}
                </p>
              )}

              <div className="h-px w-full bg-zinc-200/80 dark:bg-zinc-800/80 !mt-6" />
            </div>

            {/* Markdown Body or Fallback */}
            {docData ? (
              <div className="prose dark:prose-invert max-w-none prose-headings:font-display prose-headings:text-zinc-950 dark:prose-headings:text-white prose-p:text-zinc-700 dark:prose-p:text-zinc-300 prose-p:leading-relaxed prose-a:text-amber-600 dark:prose-a:text-amber-400 prose-strong:text-zinc-950 dark:prose-strong:text-white [&_:not(pre)>code]:text-amber-800 dark:[&_:not(pre)>code]:text-amber-300 [&_:not(pre)>code]:bg-amber-500/10 dark:[&_:not(pre)>code]:bg-amber-500/15 [&_:not(pre)>code]:px-1.5 [&_:not(pre)>code]:py-0.5 [&_:not(pre)>code]:rounded-md [&_:not(pre)>code]:before:content-none [&_:not(pre)>code]:after:content-none prose-th:text-zinc-950 dark:prose-th:text-white prose-th:bg-zinc-100/80 dark:prose-th:bg-zinc-800/60 prose-th:p-3.5 prose-td:text-zinc-700 dark:prose-td:text-zinc-300 prose-td:p-3.5 prose-tr:border-b prose-tr:border-zinc-200/80 dark:prose-tr:border-zinc-800/80 prose-li:text-zinc-800 dark:prose-li:text-zinc-200">
                <ReactMarkdown
                  remarkPlugins={[remarkGfm]}
                  components={{
                    pre: PreBlock,
                    blockquote: BlockquoteBlock,
                    table: TableBlock,
                    h2: ({ children }) => (
                      <h2 className="text-2xl font-bold font-display text-zinc-950 dark:text-white mt-10 mb-4 pb-2 border-b border-zinc-200/80 dark:border-zinc-800/80 tracking-tight flex items-center gap-2">
                        {children}
                      </h2>
                    ),
                    h3: ({ children }) => (
                      <h3 className="text-lg font-bold font-display text-zinc-950 dark:text-white mt-6 mb-3 tracking-tight">
                        {children}
                      </h3>
                    ),
                  }}
                >
                  {docData.content}
                </ReactMarkdown>
              </div>
            ) : (
              <div className="space-y-6">
                <p className="text-sm text-zinc-700 dark:text-zinc-300 leading-relaxed">
                  {Array.isArray(fallbackContent?.body)
                    ? fallbackContent.body.join('\n\n')
                    : fallbackContent?.body}
                </p>

                {fallbackContent?.list && (
                  <ul className="space-y-3 my-6">
                    {fallbackContent.list.map((item) => (
                      <li
                        key={item}
                        className="flex items-start gap-3 text-sm text-zinc-800 dark:text-zinc-200"
                      >
                        <span className="w-1.5 h-1.5 rounded-full bg-amber-500 dark:bg-amber-400 mt-2 shrink-0" />
                        <span>{item}</span>
                      </li>
                    ))}
                  </ul>
                )}

                {fallbackContent?.code && (
                  <div className="rounded-xl overflow-hidden bg-[#0a0b0e] border border-zinc-800/80 p-5 font-mono text-xs text-zinc-100 overflow-x-auto leading-relaxed shadow-inner select-text">
                    <pre>
                      {Array.isArray(fallbackContent.code)
                        ? fallbackContent.code.join('\n')
                        : fallbackContent.code}
                    </pre>
                  </div>
                )}
              </div>
            )}
          </motion.main>
        </div>
      </div>
    </div>
  )
}
