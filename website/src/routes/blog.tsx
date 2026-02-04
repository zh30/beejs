import { Calendar, User, Clock, ArrowRight, ChevronLeft } from 'lucide-react'
import { motion } from 'framer-motion'
import { Link, useParams } from 'react-router-dom'
import ReactMarkdown from 'react-markdown'
import remarkGfm from 'remark-gfm'
import { BeeLogo } from '../components/Logo'
import { useLang } from '../lib/i18n'

interface Post {
  slug: string
  title: string
  excerpt: string
  date: string
  author: string
  readTime: string
  tag: string
  content: string
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

function getPosts(): Post[] {
  try {
    return Object.entries(modules)
      .map(([path, rawContent]) => {
        const slug = path.split('/').pop()?.replace('.md', '') || 'unknown'
        if (typeof rawContent !== 'string') return null

        const { data, content } = parseFrontmatter(rawContent)

        return {
          slug,
          title: data.title || 'Untitled',
          excerpt: data.excerpt || content.slice(0, 160).replace(/[#*`]/g, '') + '...',
          date: data.date || 'Unknown Date',
          author: data.author || 'Anonymous',
          readTime: data.readTime || '1 min read',
          tag: data.tag || 'Blog',
          content,
        }
      })
      .filter((post): post is Post => post !== null)
      .sort((a, b) => new Date(b.date).getTime() - new Date(a.date).getTime())
  } catch (err) {
    console.error('Failed to process blog posts:', err)
    return []
  }
}

const allPosts = getPosts()

export default function BlogComponent() {
  const { slug } = useParams()
  const { copy } = useLang()

  if (slug) {
    const post = allPosts.find((p) => p.slug === slug)
    if (!post) return <div className="text-hud-text text-center py-24">{copy.blog.notFound}</div>
    return <BlogPostView post={post} />
  }

  return (
    <div className="relative min-h-screen bg-hud-void">
      <div className="absolute inset-0 hud-grid opacity-25 pointer-events-none" />
      <div className="scanline" />

      <div className="max-w-6xl mx-auto py-20 px-4 md:px-8 relative z-10">
        <header className="mb-16">
          <div className="flex items-center gap-4 mb-6">
            <BeeLogo className="w-12 h-12" />
            <span className="hud-tag">{copy.blog.title}</span>
          </div>
          <h1 className="text-4xl md:text-6xl font-display uppercase tracking-[0.2em]">
            {copy.blog.title}
          </h1>
          <p className="text-hud-muted mt-4 max-w-2xl">{copy.blog.subtitle}</p>
        </header>

        <div className="relative pl-6">
          <div className="absolute left-2 top-0 bottom-0 w-px bg-hud-line/70" />
          <div className="space-y-8">
            {allPosts.map((post, index) => (
              <motion.article
                key={post.slug}
                initial={{ opacity: 0, y: 12 }}
                whileInView={{ opacity: 1, y: 0 }}
                viewport={{ once: true }}
                transition={{ delay: index * 0.05 }}
                className="hud-panel-soft p-6 md:p-8 relative"
              >
                <div className="absolute left-[-14px] top-8 w-3 h-3 rounded-full border border-hud-accent bg-hud-void" />
                <div className="flex flex-wrap items-center gap-4 text-[10px] uppercase tracking-[0.35em] text-hud-muted mb-4">
                  <span className="px-3 py-1 border border-hud-line/80">{post.tag}</span>
                  <span className="flex items-center gap-2">
                    <Calendar className="w-3 h-3" /> {post.date}
                  </span>
                  <span className="flex items-center gap-2">
                    <Clock className="w-3 h-3" /> {post.readTime}
                  </span>
                </div>
                <h2 className="text-2xl md:text-3xl font-display uppercase tracking-[0.18em]">
                  <Link to={`/blog/${post.slug}`} className="hover:text-hud-accent transition-colors">
                    {post.title}
                  </Link>
                </h2>
                <p className="text-hud-muted text-sm leading-relaxed mt-4 max-w-3xl">
                  {post.excerpt}
                </p>
                <Link
                  to={`/blog/${post.slug}`}
                  className="inline-flex items-center gap-3 mt-6 text-[10px] uppercase tracking-[0.4em] text-hud-accent"
                >
                  {copy.blog.readMore} <ArrowRight className="w-4 h-4" />
                </Link>
              </motion.article>
            ))}
          </div>
        </div>
      </div>
    </div>
  )
}

function BlogPostView({ post }: { post: Post }) {
  const { copy } = useLang()

  return (
    <div className="relative min-h-screen bg-hud-void">
      <div className="absolute inset-0 hud-grid opacity-25 pointer-events-none" />
      <div className="scanline" />

      <div className="max-w-4xl mx-auto py-20 px-4 md:px-8 relative z-10">
        <header className="mb-12">
          <Link
            to="/blog"
            className="inline-flex items-center text-[10px] uppercase tracking-[0.4em] text-hud-muted hover:text-hud-text transition-colors"
          >
            <ChevronLeft className="w-4 h-4 mr-2" /> {copy.blog.back}
          </Link>

          <div className="mt-6 flex items-center gap-3 text-[10px] uppercase tracking-[0.35em] text-hud-muted">
            <span className="px-3 py-1 border border-hud-line/80">{copy.blog.tagLabel}</span>
            <span className="text-hud-accent">{post.tag}</span>
          </div>

          <h1 className="text-3xl md:text-5xl font-display uppercase tracking-[0.18em] mt-6">
            {post.title}
          </h1>

          <div className="mt-6 grid grid-cols-1 md:grid-cols-3 gap-4 text-[10px] uppercase tracking-[0.35em] text-hud-muted">
            <span className="flex items-center gap-2">
              <Calendar className="w-3 h-3" /> {copy.blog.timestamp}: {post.date}
            </span>
            <span className="flex items-center gap-2">
              <User className="w-3 h-3" /> {copy.blog.operator}: {post.author}
            </span>
            <span className="flex items-center gap-2">
              <Clock className="w-3 h-3" /> {copy.blog.readTime}: {post.readTime}
            </span>
          </div>
        </header>

        <article
          className="prose prose-invert max-w-none
            prose-headings:text-hud-text prose-headings:font-[var(--font-display)] prose-headings:uppercase prose-headings:tracking-[0.15em]
            prose-p:text-hud-muted prose-p:leading-relaxed
            prose-a:text-hud-accent hover:prose-a:text-hud-text
            prose-code:text-hud-accent prose-code:bg-hud-panel/70 prose-code:px-1.5 prose-code:rounded-sm
            prose-pre:bg-hud-panel/90 prose-pre:border prose-pre:border-hud-line/60
            prose-strong:text-hud-text
          "
        >
          <ReactMarkdown remarkPlugins={[remarkGfm]}>{post.content}</ReactMarkdown>
        </article>
      </div>
    </div>
  )
}
