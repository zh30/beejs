import { Calendar, User, Clock, ArrowRight, ChevronLeft, Info } from 'lucide-react'
import { motion } from 'framer-motion'
import { Link, useParams } from 'react-router-dom'
import ReactMarkdown from 'react-markdown'
import remarkGfm from 'remark-gfm'
import { BeeLogo } from '../components/Logo'
import { useLang, type Lang } from '../lib/i18n'

interface Post {
  slug: string
  title: string
  excerpt: string
  date: string
  author: string
  readTime: string
  tag: string
  content: string
  isFallback?: boolean
}

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

const modules = import.meta.glob('../blog/*.md', {
  query: '?raw',
  eager: true,
  import: 'default',
}) as Record<string, string>

function getPosts(lang: Lang): Post[] {
  try {
    const postsMap = new Map<string, Post>()

    // Pass 1: Look for posts matching the active language
    Object.entries(modules).forEach(([path, rawContent]) => {
      if (typeof rawContent !== 'string') return
      const filename = path.split('/').pop() || ''
      const langSuffix = `.${lang}.md`
      const isTargetLang =
        lang === 'en'
          ? !filename.includes('.zh.') &&
            !filename.includes('.es.') &&
            !filename.includes('.fr.') &&
            !filename.includes('.hi.')
          : filename.endsWith(langSuffix)
      const slug = filename.replace(/\.[a-z]{2}\.md$/, '').replace(/\.md$/, '')

      if (isTargetLang) {
        const { data, content } = parseFrontmatter(rawContent)
        postsMap.set(slug, {
          slug,
          title: data.title || 'Untitled',
          excerpt: data.excerpt || content.slice(0, 160).replace(/[#*`]/g, '') + '...',
          date: data.date || 'Unknown Date',
          author: data.author || (lang === 'zh' ? 'Beejs 团队' : 'Beejs Team'),
          readTime: data.readTime || (lang === 'zh' ? '5 分钟阅读' : '5 min read'),
          tag: data.tag || (lang === 'zh' ? '日志' : 'Blog'),
          content,
          isFallback: false,
        })
      }
    })

    // Pass 2: Gracefully fallback missing posts from English default
    Object.entries(modules).forEach(([path, rawContent]) => {
      if (typeof rawContent !== 'string') return
      const filename = path.split('/').pop() || ''
      const isEnglishFile =
        !filename.includes('.zh.') &&
        !filename.includes('.es.') &&
        !filename.includes('.fr.') &&
        !filename.includes('.hi.')
      const slug = filename.replace(/\.[a-z]{2}\.md$/, '').replace(/\.md$/, '')

      if (isEnglishFile && !postsMap.has(slug)) {
        const { data, content } = parseFrontmatter(rawContent)
        postsMap.set(slug, {
          slug,
          title: data.title || 'Untitled',
          excerpt: data.excerpt || content.slice(0, 160).replace(/[#*`]/g, '') + '...',
          date: data.date || 'Unknown Date',
          author: data.author || 'Beejs Team',
          readTime: data.readTime || '5 min read',
          tag: data.tag || 'Blog',
          content,
          isFallback: lang !== 'en',
        })
      }
    })

    return Array.from(postsMap.values()).sort(
      (a, b) => new Date(b.date).getTime() - new Date(a.date).getTime()
    )
  } catch (err) {
    console.error('Failed to process blog posts:', err)
    return []
  }
}

export default function BlogComponent() {
  const { slug } = useParams()
  const { copy, lang } = useLang()
  const posts = getPosts(lang)

  if (slug) {
    const post = posts.find((p) => p.slug === slug)
    if (!post) return <div className="text-zinc-300 text-center py-24">{copy.blog.notFound}</div>
    return <BlogPostView post={post} />
  }

  return (
    <div className="relative min-h-screen pt-10 pb-24">
      <div className="max-w-5xl mx-auto px-6 relative z-10">
        <header className="mb-14 text-center max-w-2xl mx-auto">
          <div className="inline-flex items-center gap-2 px-3.5 py-1.5 rounded-full glass-card border-amber-500/20 text-xs font-mono text-amber-400 mb-6">
            <BeeLogo className="w-4 h-4" />
            <span>{copy.blog.title}</span>
          </div>
          <h1 className="text-4xl sm:text-5xl font-extrabold text-zinc-950 dark:text-white font-display tracking-tight">
            {copy.blog.title}
          </h1>
          <p className="mt-4 text-base text-zinc-600 dark:text-zinc-400 font-normal leading-relaxed">
            {copy.blog.subtitle}
          </p>
        </header>

        <div className="grid grid-cols-1 gap-6">
          {posts.map((post, i) => (
            <motion.article
              key={post.slug}
              initial={{ opacity: 0, y: 20 }}
              animate={{ opacity: 1, y: 0 }}
              transition={{ duration: 0.3, delay: i * 0.1 }}
              className={`glass-card rounded-2xl p-8 border-zinc-200/80 dark:border-zinc-800 hover:border-amber-500/30 transition-all group ${
                i === 0 ? 'border-amber-500/40 glow-amber' : ''
              }`}
            >
              <div className="flex flex-wrap items-center gap-3 text-xs font-mono text-zinc-500 dark:text-zinc-400 mb-4">
                {i === 0 && (
                  <span className="px-2.5 py-1 rounded-full bg-amber-500 text-zinc-950 font-bold">
                    Featured
                  </span>
                )}
                <span className="px-2.5 py-1 rounded-full bg-amber-500/10 text-amber-400 border border-amber-500/20 font-semibold">
                  {post.tag}
                </span>
                <span>•</span>
                <span className="flex items-center gap-1">
                  <Calendar className="w-3.5 h-3.5" />
                  {post.date}
                </span>
                <span>•</span>
                <span className="flex items-center gap-1">
                  <Clock className="w-3.5 h-3.5" />
                  {post.readTime}
                </span>
                {post.isFallback && copy.blog.fallbackNote && (
                  <span className="px-2 py-0.5 rounded-md bg-amber-500/10 text-amber-600 dark:text-amber-400 text-[10px]">
                    English
                  </span>
                )}
              </div>

              <h2 className="text-2xl font-bold text-zinc-950 dark:text-white font-display tracking-tight group-hover:text-amber-600 dark:group-hover:text-amber-300 transition-colors">
                <Link to={`/blog/${post.slug}`}>{post.title}</Link>
              </h2>

              <p className="mt-3 text-sm text-zinc-600 dark:text-zinc-400 leading-relaxed font-normal">{post.excerpt}</p>

              <div className="mt-6 flex items-center justify-between pt-4 border-t border-zinc-200/80 dark:border-zinc-800/60">
                <span className="text-xs text-zinc-500 flex items-center gap-1 font-mono">
                  <User className="w-3.5 h-3.5" /> {post.author}
                </span>
                <Link
                  to={`/blog/${post.slug}`}
                  className="text-xs font-semibold text-amber-400 hover:text-amber-300 flex items-center gap-1 font-sans group/link"
                >
                  <span>{copy.blog.readMore}</span>
                  <ArrowRight className="w-3.5 h-3.5 transition-transform group-hover/link:translate-x-1" />
                </Link>
              </div>
            </motion.article>
          ))}
        </div>
      </div>
    </div>
  )
}

function BlogPostView({ post }: { post: Post }) {
  const { copy } = useLang()

  return (
    <div className="relative min-h-screen pt-10 pb-24">
      <div className="max-w-3xl mx-auto px-6 relative z-10">
        <Link
          to="/blog"
          className="inline-flex items-center text-xs font-mono text-zinc-500 dark:text-zinc-400 hover:text-amber-600 dark:hover:text-amber-400 transition-colors mb-8"
        >
          <ChevronLeft className="w-4 h-4 mr-1" /> {copy.blog.back}
        </Link>

        {post.isFallback && copy.blog.fallbackNote && (
          <div className="mb-6 px-4 py-3 rounded-xl border border-amber-500/30 bg-amber-500/5 text-amber-700 dark:text-amber-300 text-xs font-medium flex items-center gap-2.5">
            <Info className="w-4 h-4 shrink-0 text-amber-500" />
            <span>{copy.blog.fallbackNote}</span>
          </div>
        )}

        <article className="glass-panel rounded-2xl p-8 md:p-12 border-zinc-200/80 dark:border-zinc-800">
          <div className="flex flex-wrap items-center gap-3 text-xs font-mono text-zinc-500 mb-6">
            <span className="px-2.5 py-1 rounded-full bg-amber-500/10 text-amber-400 border border-amber-500/20 font-semibold">
              {post.tag}
            </span>
            <span>•</span>
            <span className="flex items-center gap-1">
              <Calendar className="w-3.5 h-3.5" />
              {post.date}
            </span>
            <span>•</span>
            <span className="flex items-center gap-1">
              <Clock className="w-3.5 h-3.5" />
              {post.readTime}
            </span>
          </div>

          <h1 className="text-3xl sm:text-4xl font-extrabold text-zinc-950 dark:text-white font-display tracking-tight leading-tight">
            {post.title}
          </h1>

          <div className="flex items-center gap-2 text-xs text-zinc-500 dark:text-zinc-400 font-mono mt-4 pb-8 border-b border-zinc-200/80 dark:border-zinc-800">
            <User className="w-3.5 h-3.5 text-zinc-500" />
            <span>{copy.blog.by}{post.author}</span>
          </div>

          <div className="prose dark:prose-invert max-w-none mt-8 prose-headings:font-display prose-headings:text-zinc-950 dark:prose-headings:text-white prose-p:text-zinc-700 dark:prose-p:text-zinc-300 prose-p:leading-relaxed prose-a:text-amber-600 dark:prose-a:text-amber-400 prose-code:text-amber-700 dark:prose-code:text-amber-300 prose-pre:bg-[#0a0b0e] prose-pre:border prose-pre:border-zinc-800">
            <ReactMarkdown remarkPlugins={[remarkGfm]}>{post.content}</ReactMarkdown>
          </div>
        </article>
      </div>
    </div>
  )
}
