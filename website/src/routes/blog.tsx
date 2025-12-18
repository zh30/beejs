import { Calendar, User, Clock, ArrowRight, ChevronLeft } from 'lucide-react'
import { motion } from 'framer-motion'
import { Link, useParams } from 'react-router-dom'
import ReactMarkdown from 'react-markdown'
import remarkGfm from 'remark-gfm'

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

// Use Vite's glob import to fetch all markdown files in the blog directory
const modules = import.meta.glob('../blog/*.md', { query: '?raw', eager: true, import: 'default' }) as Record<string, string>;

// Process posts from markdown files
function getPosts(): Post[] {
  try {
    return Object.entries(modules).map(([path, rawContent]) => {
      const slug = path.split('/').pop()?.replace('.md', '') || 'unknown';
      if (typeof rawContent !== 'string') {
        console.warn(`Blog content for ${path} is not a string`);
        return null;
      }

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
      .sort((a, b) => {
        const dateA = new Date(a.date).getTime();
        const dateB = new Date(b.date).getTime();
        if (isNaN(dateA)) return 1;
        if (isNaN(dateB)) return -1;
        return dateB - dateA;
      });
  } catch (err) {
    console.error('Failed to process blog posts:', err);
    return [];
  }
}

const allPosts = getPosts();

export default function BlogComponent() {
  const { slug } = useParams();

  // If a slug is provided, show the single post view
  if (slug) {
    const post = allPosts.find(p => p.slug === slug);

    if (!post) {
      return (
        <div className="max-w-3xl mx-auto py-24 px-4 text-center">
          <h1 className="text-4xl font-bold mb-6 text-white">Post Not Found</h1>
          <Link to="/blog" className="text-brand-yellow hover:underline">Return to Blog</Link>
        </div>
      );
    }

    return <BlogPostView post={post} />;
  }

  // Otherwise, show the list view
  return (
    <div className="max-w-5xl mx-auto py-24 px-4 md:px-8">
      <header className="mb-20 text-center">
        <motion.div
          initial={{ opacity: 0, scale: 0.9 }}
          animate={{ opacity: 1, scale: 1 }}
        >
          <h1 className="text-4xl md:text-6xl font-black mb-6 tracking-tighter text-white">THE <span className="text-brand-yellow">HIVE</span> BLOG</h1>
          <p className="text-gray-400 text-lg md:text-xl max-w-2xl mx-auto">
            Deep dives into systems programming, V8 internals, and the future of JavaScript.
          </p>
        </motion.div>
      </header>

      <div className="space-y-12">
        {allPosts.map((post, index) => (
          <motion.article
            key={post.slug}
            initial={{ opacity: 0, y: 20 }}
            whileInView={{ opacity: 1, y: 0 }}
            viewport={{ once: true }}
            transition={{ delay: index * 0.1 }}
            className="group glass rounded-3xl overflow-hidden flex flex-col md:flex-row hover:border-brand-yellow/30 transition-all border border-white/5"
          >
            <div className="w-full md:w-1/3 h-48 md:h-auto bg-linear-to-br from-brand-yellow/20 to-brand-gray relative flex items-center justify-center">
              <span className="text-brand-yellow/30 font-black text-6xl select-none">B</span>
              <div className="absolute top-4 left-4">
                <span className="px-3 py-1 bg-brand-black/50 backdrop-blur-md rounded-full text-[10px] uppercase font-bold text-brand-yellow border border-brand-yellow/20">
                  {post.tag}
                </span>
              </div>
            </div>

            <div className="p-8 md:p-12 grow flex flex-col justify-between">
              <div>
                <div className="flex items-center space-x-4 text-xs text-gray-500 mb-6">
                  <span className="flex items-center"><Calendar className="w-3 h-3 mr-1" /> {post.date}</span>
                  <span className="flex items-center"><User className="w-3 h-3 mr-1" /> {post.author}</span>
                  <span className="flex items-center"><Clock className="w-3 h-3 mr-1" /> {post.readTime}</span>
                </div>
                <h2 className="text-2xl md:text-3xl font-bold mb-4 group-hover:text-brand-yellow transition-colors leading-tight text-white">
                  <Link to={`/blog/${post.slug}`}>{post.title}</Link>
                </h2>
                <p className="text-gray-400 text-sm md:text-base leading-relaxed mb-8">
                  {post.excerpt}
                </p>
              </div>

              <Link
                to={`/blog/${post.slug}`}
                className="inline-flex items-center text-brand-yellow font-bold text-sm tracking-wide group-hover:translate-x-1 transition-transform uppercase"
              >
                READ ARTICLE <ArrowRight className="w-4 h-4 ml-2" />
              </Link>
            </div>
          </motion.article>
        ))}
      </div>
    </div>
  )
}



function BlogPostView({ post }: { post: Post }) {
  return (
    <div className="max-w-4xl mx-auto py-24 px-4 md:px-8">
      <Link to="/blog" className="inline-flex items-center text-gray-500 hover:text-white mb-12 transition-colors">
        <ChevronLeft className="w-4 h-4 mr-2" /> Back to Blog
      </Link>

      <article>
        <header className="mb-12">
          <div className="flex items-center space-x-2 text-brand-yellow mb-6">
            <span className="px-3 py-1 bg-brand-yellow/10 rounded-full text-[10px] uppercase font-bold border border-brand-yellow/20">
              {post.tag}
            </span>
          </div>
          <h1 className="text-3xl md:text-6xl font-black mb-8 tracking-tighter text-white leading-tight">
            {post.title}
          </h1>
          <div className="flex flex-wrap items-center gap-6 text-sm text-gray-500 pb-12 border-b border-white/10">
            <div className="flex items-center"><Calendar className="w-4 h-4 mr-2 text-brand-yellow" /> {post.date}</div>
            <div className="flex items-center"><User className="w-4 h-4 mr-2 text-brand-yellow" /> {post.author}</div>
            <div className="flex items-center"><Clock className="w-4 h-4 mr-2 text-brand-yellow" /> {post.readTime}</div>
          </div>
        </header>

        <div className="prose prose-invert prose-brand max-w-none text-gray-300 leading-relaxed 
          prose-headings:text-white prose-headings:font-bold prose-headings:tracking-tight
          prose-a:text-brand-yellow hover:prose-a:text-yellow-400
          prose-code:text-brand-yellow prose-code:bg-white/5 prose-code:px-1 prose-code:rounded
          prose-pre:bg-black/50 prose-pre:border prose-pre:border-white/10 prose-pre:rounded-2xl
          prose-strong:text-white
        ">
          <ReactMarkdown remarkPlugins={[remarkGfm]}>
            {post.content}
          </ReactMarkdown>
        </div>
      </article>

      <div className="mt-24 pt-12 border-t border-white/10 flex justify-between items-center">
        <div className="flex items-center space-x-4">
          <div className="w-12 h-12 rounded-full bg-brand-yellow flex items-center justify-center text-brand-black font-bold">
            {post.author[0]}
          </div>
          <div>
            <p className="text-white font-bold">{post.author}</p>
            <p className="text-xs text-gray-500">Core Contributor</p>
          </div>
        </div>
        <button className="px-6 py-2 glass rounded-lg text-sm text-gray-400 hover:text-white transition-colors">
          Share Article
        </button>
      </div>
    </div>
  )
}
