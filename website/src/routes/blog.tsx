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
          <h1 className="text-4xl sm:text-5xl font-extrabold text-white font-display tracking-tight">
            {copy.blog.title}
          </h1>
          <p className="mt-4 text-base text-zinc-400 font-normal leading-relaxed">
            {copy.blog.subtitle}
          </p>
        </header>

        <div className="grid grid-cols-1 gap-6">
          {allPosts.map((post, i) => (
            <motion.article
              key={post.slug}
              initial={{ opacity: 0, y: 20 }}
              animate={{ opacity: 1, y: 0 }}
              transition={{ duration: 0.3, delay: i * 0.1 }}
              className="glass-card rounded-2xl p-8 border-zinc-800 hover:border-amber-500/30 transition-all group"
            >
              <div className="flex flex-wrap items-center gap-3 text-xs font-mono text-zinc-500 mb-4">
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

              <h2 className="text-2xl font-bold text-white font-display tracking-tight group-hover:text-amber-300 transition-colors">
                <Link to={`/blog/${post.slug}`}>{post.title}</Link>
              </h2>

              <p className="mt-3 text-sm text-zinc-400 leading-relaxed font-normal">{post.excerpt}</p>

              <div className="mt-6 flex items-center justify-between pt-4 border-t border-zinc-800/60">
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
          className="inline-flex items-center text-xs font-mono text-zinc-400 hover:text-amber-400 transition-colors mb-8"
        >
          <ChevronLeft className="w-4 h-4 mr-1" /> {copy.blog.back}
        </Link>

        <article className="glass-panel rounded-2xl p-8 md:p-12 border-zinc-800">
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

          <h1 className="text-3xl sm:text-4xl font-extrabold text-white font-display tracking-tight leading-tight">
            {post.title}
          </h1>

          <div className="flex items-center gap-2 text-xs text-zinc-400 font-mono mt-4 pb-8 border-b border-zinc-800">
            <User className="w-3.5 h-3.5 text-zinc-500" />
            <span>By {post.author}</span>
          </div>

          <div className="prose prose-invert max-w-none mt-8 prose-headings:font-display prose-headings:text-white prose-p:text-zinc-300 prose-p:leading-relaxed prose-a:text-amber-400 prose-code:text-amber-300 prose-pre:bg-[#0a0b0e] prose-pre:border prose-pre:border-zinc-800">
            <ReactMarkdown remarkPlugins={[remarkGfm]}>{post.content}</ReactMarkdown>
          </div>
        </article>
      </div>
    </div>
  )
}
