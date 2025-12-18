import { Calendar, User, Clock, ArrowRight, ChevronLeft } from 'lucide-react'
import { motion } from 'framer-motion'
import { Link, useParams } from 'react-router-dom'
import ReactMarkdown from 'react-markdown'
import remarkGfm from 'remark-gfm'
import { BeeLogo } from '../components/Logo'

interface Post {
  slug: string;
  title: string;
  excerpt: string;
  date: string;
  author: string;
  readTime: string;
  tag: string;
  content: string;
}

// Basic browser-safe frontmatter parser
function parseFrontmatter(raw: string): { data: Record<string, string>, content: string } {
  const match = raw.match(/^---\s*([\s\S]*?)\s*---\s*([\s\S]*)$/);
  if (!match) return { data: {}, content: raw };

  const yaml = match[1];
  const content = match[2];
  const data: Record<string, string> = {};

  yaml.split('\n').filter(Boolean).forEach(line => {
    const [key, ...valueParts] = line.split(':');
    if (key && valueParts.length > 0) {
      data[key.trim()] = valueParts.join(':').trim().replace(/^["']|["']$/g, '');
    }
  });

  return { data, content };
}

const modules = import.meta.glob('../blog/*.md', { query: '?raw', eager: true, import: 'default' }) as Record<string, string>;

function getPosts(): Post[] {
  try {
    return Object.entries(modules).map(([path, rawContent]) => {
      const slug = path.split('/').pop()?.replace('.md', '') || 'unknown';
      if (typeof rawContent !== 'string') return null;

      const { data, content } = parseFrontmatter(rawContent);

      return {
        slug,
        title: data.title || 'Untitled',
        excerpt: data.excerpt || (content.slice(0, 160).replace(/[#*`]/g, '') + '...'),
        date: data.date || 'Unknown Date',
        author: data.author || 'Anonymous',
        readTime: data.readTime || '1 min read',
        tag: data.tag || 'Blog',
        content: content
      };
    })
      .filter((post): post is Post => post !== null)
      .sort((a, b) => new Date(b.date).getTime() - new Date(a.date).getTime());
  } catch (err) {
    console.error('Failed to process blog posts:', err);
    return [];
  }
}

const allPosts = getPosts();

export default function BlogComponent() {
  const { slug } = useParams();

  if (slug) {
    const post = allPosts.find(p => p.slug === slug);
    if (!post) return <div className="text-white text-center py-24">Post Not Found</div>;
    return <BlogPostView post={post} />;
  }

  return (
    <div className="relative min-h-screen bg-brand-black">
      {/* Background Decor */}
      <div className="absolute inset-0 cyber-grid opacity-20 pointer-events-none" />
      <div className="scanline opacity-5 pointer-events-none" />

      <div className="max-w-5xl mx-auto py-24 px-4 md:px-8 relative z-10">
        <header className="mb-24 text-center">
          <motion.div
            initial={{ opacity: 0, scale: 0.9 }}
            animate={{ opacity: 1, scale: 1 }}
          >
            <div className="flex justify-center mb-10 text-brand-yellow">
              <BeeLogo className="w-16 h-16" />
            </div>
            <h1 className="text-5xl md:text-8xl font-black mb-6 tracking-tighter text-white uppercase italic">
              The <span className="text-brand-yellow glow-text">Hive</span> Terminal
            </h1>
            <p className="text-gray-500 text-sm md:text-base max-w-xl mx-auto font-mono uppercase tracking-[0.2em]">
              Synchronizing deep-space telemetry, V8 internals, and the future of JavaScript.
            </p>
          </motion.div>
        </header>

        <div className="space-y-10">
          {allPosts.map((post, index) => (
            <motion.article
              key={post.slug}
              initial={{ opacity: 0, x: -20 }}
              whileInView={{ opacity: 1, x: 0 }}
              viewport={{ once: true }}
              transition={{ delay: index * 0.1 }}
              className="group relative"
            >
              <div className="absolute inset-0 bg-brand-yellow/5 -skew-x-6 transform group-hover:bg-brand-yellow/10 transition-all duration-300 pointer-events-none" />
              <div className="relative glass border border-white/5 p-8 md:p-12 flex flex-col md:flex-row gap-10 items-center">
                <div className="w-full md:w-32 h-32 shrink-0 flex items-center justify-center bg-brand-yellow/10 relative overflow-hidden group-hover:bg-brand-yellow/20 transition-all">
                  <span className="text-brand-yellow font-black text-4xl italic group-hover:scale-110 transition-transform">B</span>
                </div>

                <div className="grow">
                  <div className="flex flex-wrap items-center gap-4 text-[10px] font-black uppercase tracking-widest text-gray-500 mb-6">
                    <span className="px-2 py-0.5 bg-brand-yellow text-brand-black">{post.tag}</span>
                    <span className="flex items-center"><Calendar className="w-3 h-3 mr-1" /> {post.date}</span>
                    <span className="flex items-center"><Clock className="w-3 h-3 mr-1" /> {post.readTime}</span>
                  </div>

                  <h2 className="text-2xl md:text-4xl font-black mb-4 text-white group-hover:text-brand-yellow transition-colors uppercase italic tracking-tighter">
                    <Link to={`/blog/${post.slug}`}>{post.title}</Link>
                  </h2>
                  <p className="text-gray-400 text-sm leading-relaxed max-w-2xl mb-8 font-mono">
                    {post.excerpt}
                  </p>

                  <Link
                    to={`/blog/${post.slug}`}
                    className="inline-flex items-center text-xs font-black text-brand-yellow uppercase tracking-[0.3em] group-hover:translate-x-2 transition-transform"
                  >
                    ESTABLISH LINK <ArrowRight className="w-4 h-4 ml-2" />
                  </Link>
                </div>
              </div>
            </motion.article>
          ))}
        </div>
      </div>
    </div>
  )
}

function BlogPostView({ post }: { post: Post }) {
  return (
    <div className="relative min-h-screen bg-brand-black">
      {/* Background Decor */}
      <div className="absolute inset-0 cyber-grid opacity-20 pointer-events-none" />
      <div className="scanline opacity-5 pointer-events-none" />

      <div className="max-w-4xl mx-auto py-24 px-4 md:px-8 relative z-10">
        <header className="mb-20">
          <Link to="/blog" className="inline-flex items-center text-xs font-black text-gray-500 hover:text-brand-yellow transition-colors uppercase tracking-[0.2em] mb-12">
            <ChevronLeft className="w-4 h-4 mr-2" /> Return to Terminal
          </Link>

          <div className="flex items-center space-x-2 text-brand-yellow mb-8">
            <span className="px-4 py-1 bg-brand-yellow/10 text-[10px] font-black uppercase tracking-widest border border-brand-yellow/20">
              MANIFEST: {post.tag}
            </span>
          </div>

          <h1 className="text-4xl md:text-7xl font-black mb-10 text-white uppercase italic tracking-tighter leading-none">
            {post.title.split(' ').map((word, i) => (
              <span key={i} className={i % 2 !== 0 ? 'text-brand-yellow glow-text' : ''}>
                {word}{' '}
              </span>
            ))}
          </h1>

          <div className="flex flex-wrap items-center gap-8 py-8 border-y border-white/5 font-mono text-[10px] text-gray-500 uppercase tracking-widest">
            <div className="flex items-center"><Calendar className="w-4 h-4 mr-2 text-brand-yellow" /> TIMESTAMP: {post.date}</div>
            <div className="flex items-center"><User className="w-4 h-4 mr-2 text-brand-yellow" /> OPERATOR: {post.author}</div>
            <div className="flex items-center"><Clock className="w-4 h-4 mr-2 text-brand-yellow" /> SYNC TIME: {post.readTime}</div>
          </div>
        </header>

        <article className="prose prose-invert prose-brand max-w-none 
          prose-headings:text-white prose-headings:font-black prose-headings:uppercase prose-headings:italic prose-headings:tracking-tighter
          prose-p:text-gray-400 prose-p:leading-relaxed prose-p:font-mono
          prose-a:text-brand-yellow hover:prose-a:text-white transition-colors
          prose-code:text-brand-yellow prose-code:bg-white/5 prose-code:px-1.5 prose-code:rounded-sm
          prose-pre:bg-black/80 prose-pre:border prose-pre:border-white/10 prose-pre:rounded-none
          prose-strong:text-white prose-img:rounded-none
        ">
          <ReactMarkdown remarkPlugins={[remarkGfm]}>
            {post.content}
          </ReactMarkdown>
        </article>

        <div className="mt-32 pt-16 border-t border-white/10 flex flex-col md:flex-row justify-between items-center gap-10">
          <div className="flex items-center space-x-6">
            <div className="w-16 h-16 bg-brand-yellow flex items-center justify-center text-brand-black transition-transform hover:rotate-12">
              <BeeLogo className="w-10 h-10 text-brand-black shadow-none drop-shadow-none" />
            </div>
            <div>
              <p className="text-white font-black uppercase italic tracking-tighter text-xl">{post.author}</p>
              <p className="text-[10px] text-gray-500 uppercase tracking-widest font-mono">Hive Core Contributor</p>
            </div>
          </div>
          <button className="px-10 py-5 bg-white text-brand-black font-black uppercase -skew-x-12 hover:bg-brand-yellow transition-all">
            <span className="inline-block skew-x-12">Broadcast Feed</span>
          </button>
        </div>
      </div>
    </div>
  )
}
