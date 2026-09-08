import { Link, useParams } from 'react-router-dom'
import { motion } from 'framer-motion'
import { useState, useEffect, useMemo, type ReactNode } from 'react'
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
  ArrowRight,
  Sparkles,
  Package,
  Copy,
  Check,
  Search,
  X,
  Workflow,
  CheckCircle2,
  Box,
  Gauge,
  ShieldCheck,
  Binary,
  FileCode,
  Info,
  Lightbulb,
  AlertCircle,
  AlertTriangle,
  ListTree,
  Database,
  Rocket,
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
  'embedded-db': <Database className="w-4 h-4" />,
  'standard-library': <Sparkles className="w-4 h-4" />,
  'package-manager-dlx': <Binary className="w-4 h-4" />,
  'deployment-docker': <Rocket className="w-4 h-4" />,
  'ide-extension': <FileCode className="w-4 h-4" />,
  'task-runner': <Workflow className="w-4 h-4" />,
  'code-quality': <CheckCircle2 className="w-4 h-4" />,
  'bundling-compilation': <Box className="w-4 h-4" />,
  'testing-benchmarking': <Gauge className="w-4 h-4" />,
  'debugging-lsp': <Terminal className="w-4 h-4" />,
  'agent-sandbox': <ShieldCheck className="w-4 h-4" />,
  'import-maps-native': <Binary className="w-4 h-4" />,
  'types-lsp': <FileCode className="w-4 h-4" />,
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

function slugify(text: string): string {
  return text
    .toLowerCase()
    .replace(/[^\w\u4e00-\u9fa5]+/g, '-')
    .replace(/^-+|-+$/g, '')
}

function extractToc(content: string): { id: string; title: string; level: number }[] {
  const headingRegex = /^(#{2,3})\s+(.+)$/gm
  const toc: { id: string; title: string; level: number }[] = []
  let match
  while ((match = headingRegex.exec(content)) !== null) {
    const level = match[1].length
    const title = match[2].trim().replace(/[*`_]/g, '')
    const id = slugify(title)
    if (id) {
      toc.push({ id, title, level })
    }
  }
  return toc
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
          <span>code</span>
        </span>
        <button
          onClick={handleCopy}
          aria-label="Copy code"
          className="inline-flex items-center gap-1 text-[11px] text-zinc-400 hover:text-white px-2 py-0.5 rounded transition-colors bg-zinc-800/50 hover:bg-zinc-700/50 cursor-pointer"
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
  const extractText = (node: any): string => {
    if (typeof node === 'string') return node
    if (Array.isArray(node)) return node.map(extractText).join('')
    if (node?.props?.children) return extractText(node.props.children)
    return ''
  }

  const text = extractText(children).trim()

  if (text.includes('[!NOTE]')) {
    return (
      <div className="my-6 rounded-2xl border-l-4 border-sky-500 bg-sky-500/10 dark:bg-sky-500/15 p-4 sm:p-5 text-sm text-sky-950 dark:text-sky-100 shadow-sm leading-relaxed">
        <div className="flex items-center gap-2 font-semibold text-sky-600 dark:text-sky-400 mb-1">
          <Info className="w-4 h-4" /> NOTE
        </div>
        <div className="[&>p]:m-0">{children}</div>
      </div>
    )
  }
  if (text.includes('[!TIP]')) {
    return (
      <div className="my-6 rounded-2xl border-l-4 border-emerald-500 bg-emerald-500/10 dark:bg-emerald-500/15 p-4 sm:p-5 text-sm text-emerald-950 dark:text-emerald-100 shadow-sm leading-relaxed">
        <div className="flex items-center gap-2 font-semibold text-emerald-600 dark:text-emerald-400 mb-1">
          <Lightbulb className="w-4 h-4" /> TIP
        </div>
        <div className="[&>p]:m-0">{children}</div>
      </div>
    )
  }
  if (text.includes('[!IMPORTANT]')) {
    return (
      <div className="my-6 rounded-2xl border-l-4 border-amber-500 bg-amber-500/10 dark:bg-amber-500/15 p-4 sm:p-5 text-sm text-amber-950 dark:text-amber-100 shadow-sm leading-relaxed">
        <div className="flex items-center gap-2 font-semibold text-amber-600 dark:text-amber-400 mb-1">
          <AlertCircle className="w-4 h-4" /> IMPORTANT
        </div>
        <div className="[&>p]:m-0">{children}</div>
      </div>
    )
  }
  if (text.includes('[!WARNING]') || text.includes('[!CAUTION]')) {
    return (
      <div className="my-6 rounded-2xl border-l-4 border-rose-500 bg-rose-500/10 dark:bg-rose-500/15 p-4 sm:p-5 text-sm text-rose-950 dark:text-rose-100 shadow-sm leading-relaxed">
        <div className="flex items-center gap-2 font-semibold text-rose-600 dark:text-rose-400 mb-1">
          <AlertTriangle className="w-4 h-4" /> WARNING
        </div>
        <div className="[&>p]:m-0">{children}</div>
      </div>
    )
  }

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

  const [searchQuery, setSearchQuery] = useState('')
  const [activeHeading, setActiveHeading] = useState<string>('')

  const docData = getDocContent(section, lang)

  // Extract table of contents from markdown content
  const toc = useMemo(() => {
    return docData?.content ? extractToc(docData.content) : []
  }, [docData?.content])

  // Track active heading on scroll
  useEffect(() => {
    const handleScroll = () => {
      const headings = document.querySelectorAll('h2[id], h3[id]')
      let current = ''
      for (const el of Array.from(headings)) {
        const top = el.getBoundingClientRect().top
        if (top <= 150) {
          current = el.id
        }
      }
      setActiveHeading(current)
    }

    window.addEventListener('scroll', handleScroll, { passive: true })
    handleScroll()
    return () => window.removeEventListener('scroll', handleScroll)
  }, [section])

  // Find active group title
  let currentGroupTitle = manual.title
  for (const group of manual.groups) {
    if (group.items.some((item) => item.id === section)) {
      currentGroupTitle = group.title
      break
    }
  }

  // Flatten items for pagination
  const allItems = useMemo(() => {
    return manual.groups.flatMap((g) => g.items)
  }, [manual.groups])

  const currentIndex = allItems.findIndex((item) => item.id === section)
  const prevItem = currentIndex > 0 ? allItems[currentIndex - 1] : null
  const nextItem = currentIndex >= 0 && currentIndex < allItems.length - 1 ? allItems[currentIndex + 1] : null

  // Filter groups according to search query
  const filteredGroups = useMemo(() => {
    if (!searchQuery.trim()) return manual.groups
    const q = searchQuery.toLowerCase().trim()
    return manual.groups
      .map((g) => ({
        ...g,
        items: g.items.filter(
          (item) => item.label.toLowerCase().includes(q) || item.id.toLowerCase().includes(q)
        ),
      }))
      .filter((g) => g.items.length > 0)
  }, [manual.groups, searchQuery])

  // Fallback content if no markdown found
  const fallbackContent =
    manual.sections[section as keyof typeof manual.sections] || manual.sections.introduction

  const title = docData?.data?.title || fallbackContent?.title || section
  const subtitle = docData?.data?.subtitle || fallbackContent?.subtitle || ''

  return (
    <div className="relative min-h-screen pt-8 pb-24">
      <div className="max-w-[1360px] mx-auto px-4 sm:px-6 relative z-10">
        <div className="grid grid-cols-1 lg:grid-cols-[280px_1fr] xl:grid-cols-[280px_1fr_220px] gap-8 xl:gap-10">
          
          {/* Left Sidebar */}
          <aside className="glass-panel rounded-2xl p-5 h-fit sticky top-24 border-zinc-200/80 dark:border-zinc-800">
            <Link
              to="/"
              className="inline-flex items-center text-xs font-mono text-zinc-600 dark:text-zinc-400 hover:text-amber-600 dark:hover:text-amber-400 transition-colors mb-5"
            >
              <ArrowLeft className="w-3.5 h-3.5 mr-2" /> {manual.backToHome}
            </Link>

            {/* Quick Search Filter */}
            <div className="relative mb-6">
              <Search className="w-3.5 h-3.5 text-zinc-400 absolute left-3 top-1/2 -translate-y-1/2 pointer-events-none" />
              <input
                type="text"
                value={searchQuery}
                onChange={(e) => setSearchQuery(e.target.value)}
                placeholder={manual.searchPlaceholder || 'Search docs...'}
                className="w-full pl-8.5 pr-8 py-2 text-xs rounded-xl bg-zinc-100 dark:bg-zinc-900/90 border border-zinc-200 dark:border-zinc-800 text-zinc-900 dark:text-zinc-100 placeholder-zinc-400 focus:outline-none focus:border-amber-500/80 transition-colors"
              />
              {searchQuery && (
                <button
                  onClick={() => setSearchQuery('')}
                  aria-label="Clear search"
                  className="absolute right-2.5 top-1/2 -translate-y-1/2 text-zinc-400 hover:text-zinc-600 dark:hover:text-zinc-200 p-0.5 cursor-pointer"
                >
                  <X className="w-3.5 h-3.5" />
                </button>
              )}
            </div>

            {/* Nav Groups */}
            <div className="space-y-6 max-h-[calc(100vh-220px)] overflow-y-auto pr-1">
              {filteredGroups.map((group) => (
                <div key={group.title}>
                  <h4 className="text-[11px] font-mono uppercase tracking-wider text-zinc-600 dark:text-zinc-400 mb-2.5 font-semibold">
                    {group.title}
                  </h4>
                  <div className="space-y-1">
                    {group.items.map((item) => (
                      <Link
                        key={item.id}
                        to={`/docs/${item.id}`}
                        className={`flex items-center justify-between px-3 py-2 rounded-xl text-xs font-medium transition-all ${
                          section === item.id
                            ? 'text-amber-700 dark:text-white bg-amber-500/10 border border-amber-500/30 shadow-sm font-semibold'
                            : 'text-zinc-700 dark:text-zinc-400 hover:text-zinc-950 dark:hover:text-zinc-200 hover:bg-zinc-200/60 dark:hover:bg-zinc-800/40'
                        }`}
                      >
                        <div className="flex items-center gap-2.5 truncate">
                          <span
                            className={
                              section === item.id
                                ? 'text-amber-600 dark:text-amber-400'
                                : 'text-zinc-500 dark:text-zinc-500'
                            }
                          >
                            {iconMap[item.id] || <Book className="w-4 h-4" />}
                          </span>
                          <span className="truncate">{item.label}</span>
                        </div>

                        {item.badge && (
                          <span
                            className={`ml-2 text-[9px] font-mono px-1.5 py-0.5 rounded-full font-semibold shrink-0 ${
                              item.badge === 'NEW'
                                ? 'bg-amber-500/20 text-amber-600 dark:text-amber-400 border border-amber-500/30'
                                : item.badge === 'Agent'
                                ? 'bg-purple-500/20 text-purple-600 dark:text-purple-400 border border-purple-500/30'
                                : item.badge === 'AI'
                                ? 'bg-blue-500/20 text-blue-600 dark:text-blue-400 border border-blue-500/30'
                                : 'bg-emerald-500/20 text-emerald-600 dark:text-emerald-400 border border-emerald-500/30'
                            }`}
                          >
                            {item.badge}
                          </span>
                        )}
                      </Link>
                    ))}
                  </div>
                </div>
              ))}

              {filteredGroups.length === 0 && (
                <div className="text-xs text-zinc-500 text-center py-6">
                  No matching documentation pages.
                </div>
              )}
            </div>
          </aside>

          {/* Main Doc Content */}
          <motion.main
            key={section}
            initial={{ opacity: 0, y: 16 }}
            animate={{ opacity: 1, y: 0 }}
            transition={{ duration: 0.3 }}
            className="glass-panel rounded-2xl p-6 sm:p-10 md:p-12 border-zinc-200/80 dark:border-zinc-800 min-w-0"
          >
            {/* Header / Breadcrumb */}
            <div className="space-y-4 mb-8">
              <div className="flex items-center gap-2 text-xs font-mono text-amber-700 dark:text-amber-400 flex-wrap">
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
                    h2: ({ children }) => {
                      const id = slugify(String(children))
                      return (
                        <h2
                          id={id}
                          className="text-2xl font-bold font-display text-zinc-950 dark:text-white mt-10 mb-4 pb-2 border-b border-zinc-200/80 dark:border-zinc-800/80 tracking-tight flex items-center gap-2 group scroll-mt-24"
                        >
                          <a href={`#${id}`} className="hover:underline text-inherit no-underline">
                            {children}
                          </a>
                        </h2>
                      )
                    },
                    h3: ({ children }) => {
                      const id = slugify(String(children))
                      return (
                        <h3
                          id={id}
                          className="text-lg font-bold font-display text-zinc-950 dark:text-white mt-6 mb-3 tracking-tight scroll-mt-24"
                        >
                          <a href={`#${id}`} className="hover:underline text-inherit no-underline">
                            {children}
                          </a>
                        </h3>
                      )
                    },
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

            {/* Pagination Cards (Previous / Next) */}
            <div className="mt-14 pt-8 border-t border-zinc-200/80 dark:border-zinc-800/80 grid grid-cols-1 sm:grid-cols-2 gap-4">
              {prevItem ? (
                <Link
                  to={`/docs/${prevItem.id}`}
                  className="group flex flex-col p-4 rounded-xl border border-zinc-200/80 dark:border-zinc-800 bg-white/40 dark:bg-zinc-900/40 hover:border-amber-500/50 hover:bg-amber-500/5 transition-all"
                >
                  <span className="text-[10px] font-mono uppercase tracking-wider text-zinc-500 flex items-center gap-1 mb-1">
                    <ArrowLeft className="w-3 h-3 group-hover:-translate-x-1 transition-transform" />
                    {manual.previousPage || 'Previous'}
                  </span>
                  <span className="text-sm font-semibold text-zinc-900 dark:text-white group-hover:text-amber-600 dark:group-hover:text-amber-400 transition-colors truncate">
                    {prevItem.label}
                  </span>
                </Link>
              ) : (
                <div />
              )}

              {nextItem ? (
                <Link
                  to={`/docs/${nextItem.id}`}
                  className="group flex flex-col p-4 rounded-xl border border-zinc-200/80 dark:border-zinc-800 bg-white/40 dark:bg-zinc-900/40 hover:border-amber-500/50 hover:bg-amber-500/5 transition-all text-right items-end"
                >
                  <span className="text-[10px] font-mono uppercase tracking-wider text-zinc-500 flex items-center gap-1 mb-1">
                    {manual.nextPage || 'Next'}
                    <ArrowRight className="w-3 h-3 group-hover:translate-x-1 transition-transform" />
                  </span>
                  <span className="text-sm font-semibold text-zinc-900 dark:text-white group-hover:text-amber-600 dark:group-hover:text-amber-400 transition-colors truncate">
                    {nextItem.label}
                  </span>
                </Link>
              ) : (
                <div />
              )}
            </div>
          </motion.main>

          {/* Right Sidebar: On This Page (TOC) */}
          <aside className="hidden xl:block">
            <div className="sticky top-24 space-y-4">
              <div className="glass-panel rounded-2xl p-5 border-zinc-200/80 dark:border-zinc-800">
                <div className="flex items-center gap-2 text-xs font-mono uppercase tracking-wider text-zinc-600 dark:text-zinc-400 mb-3 font-semibold">
                  <ListTree className="w-3.5 h-3.5 text-amber-500" />
                  <span>{manual.onThisPage || 'On this page'}</span>
                </div>

                {toc.length > 0 ? (
                  <nav className="space-y-1 max-h-[calc(100vh-250px)] overflow-y-auto text-xs">
                    {toc.map((item) => (
                      <a
                        key={item.id}
                        href={`#${item.id}`}
                        className={`block py-1 px-2 rounded-lg transition-colors truncate ${
                          item.level === 3 ? 'pl-4 text-[11px]' : 'font-medium'
                        } ${
                          activeHeading === item.id
                            ? 'text-amber-600 dark:text-amber-400 bg-amber-500/10 font-semibold'
                            : 'text-zinc-600 dark:text-zinc-400 hover:text-zinc-950 dark:hover:text-zinc-200'
                        }`}
                      >
                        {item.title}
                      </a>
                    ))}
                  </nav>
                ) : (
                  <div className="text-xs text-zinc-500 italic">No headings on this page.</div>
                )}
              </div>

              {/* Quick Links Card */}
              <div className="glass-panel rounded-2xl p-5 border-zinc-200/80 dark:border-zinc-800 text-xs text-zinc-600 dark:text-zinc-400 space-y-2">
                <div className="font-semibold text-zinc-900 dark:text-zinc-200">Beejs Ecosystem</div>
                <p className="text-[11px] leading-relaxed">
                  Fast, lightweight, native AI & Agent-grade JS/TS runtime written in Rust & V8.
                </p>
                <div className="pt-2 flex items-center gap-2">
                  <a
                    href="https://github.com/zh30/beejs"
                    target="_blank"
                    rel="noreferrer"
                    className="inline-flex items-center gap-1 text-[11px] text-amber-600 dark:text-amber-400 hover:underline"
                  >
                    GitHub Repository →
                  </a>
                </div>
              </div>
            </div>
          </aside>

        </div>
      </div>
    </div>
  )
}
