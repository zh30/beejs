import { createFileRoute, Link } from '@tanstack/react-router'
import { Calendar, User, Clock, ArrowRight } from 'lucide-react'
import { motion } from 'framer-motion'

export const Route = createFileRoute('/blog')({
  component: BlogComponent,
})

function BlogComponent() {
  const posts = [
    {
      id: '1',
      title: 'Announcing Beejs v0.1.0: The Fastest JS Runtime for AI',
      excerpt: 'We are thrilled to announce the initial release of Beejs, a runtime designed from the ground up for massive concurrency and ultra-fast startup.',
      date: 'Dec 15, 2025',
      author: 'Henry',
      readTime: '5 min read',
      tag: 'Announcement'
    },
    {
      id: '2',
      title: 'How We Achieved 11ms Startup Time with Isolate Pooling',
      excerpt: 'Startup time is critical for serverless and AI agents. Learn about our unique approach to V8 isolate management.',
      date: 'Dec 12, 2025',
      author: 'Beejs Team',
      readTime: '12 min read',
      tag: 'Engineering'
    },
    {
      id: '3',
      title: 'Benchmarking Beejs vs Bun: A Deep Dive',
      excerpt: 'A comprehensive performance comparison across various real-world scenarios, from small script execution to high-load web servers.',
      date: 'Dec 10, 2025',
      author: 'Henry',
      readTime: '8 min read',
      tag: 'Performance'
    }
  ]

  return (
    <div className="max-w-5xl mx-auto py-24 px-4 md:px-8">
      <header className="mb-20 text-center">
        <h1 className="text-4xl md:text-6xl font-black mb-6 tracking-tighter">THE <span className="text-brand-yellow">HIVE</span> BLOG</h1>
        <p className="text-gray-400 text-lg md:text-xl max-w-2xl mx-auto">
          Deep dives into systems programming, V8 internals, and the future of JavaScript.
        </p>
      </header>

      <div className="space-y-12">
        {posts.map((post, index) => (
          <motion.article
            key={post.id}
            initial={{ opacity: 0, y: 20 }}
            animate={{ opacity: 1, y: 0 }}
            transition={{ delay: index * 0.1 }}
            className="group glass rounded-3xl overflow-hidden flex flex-col md:flex-row hover:border-brand-yellow/30 transition-all border border-white/5"
          >
            <div className="w-full md:w-1/3 h-48 md:h-auto bg-gradient-to-br from-brand-yellow/20 to-brand-gray relative flex items-center justify-center">
              <span className="text-brand-yellow/30 font-black text-6xl select-none">B</span>
              <div className="absolute top-4 left-4">
                <span className="px-3 py-1 bg-brand-black/50 backdrop-blur-md rounded-full text-[10px] uppercase font-bold text-brand-yellow border border-brand-yellow/20">
                  {post.tag}
                </span>
              </div>
            </div>

            <div className="p-8 md:p-12 flex-grow flex flex-col justify-between">
              <div>
                <div className="flex items-center space-x-4 text-xs text-gray-500 mb-6">
                  <span className="flex items-center"><Calendar className="w-3 h-3 mr-1" /> {post.date}</span>
                  <span className="flex items-center"><User className="w-3 h-3 mr-1" /> {post.author}</span>
                  <span className="flex items-center"><Clock className="w-3 h-3 mr-1" /> {post.readTime}</span>
                </div>
                <h2 className="text-2xl md:text-3xl font-bold mb-4 group-hover:text-brand-yellow transition-colors leading-tight">
                  <Link to="/blog">{post.title}</Link>
                </h2>
                <p className="text-gray-400 text-sm md:text-base leading-relaxed mb-8">
                  {post.excerpt}
                </p>
              </div>

              <Link to="/blog" className="inline-flex items-center text-brand-yellow font-bold text-sm tracking-wide group-hover:translate-x-1 transition-transform">
                READ ARTICLE <ArrowRight className="w-4 h-4 ml-2" />
              </Link>
            </div>
          </motion.article>
        ))}
      </div>

      <div className="mt-20 text-center">
        <button className="px-8 py-3 glass rounded-xl text-gray-400 hover:text-white transition-colors">
          View All Posts
        </button>
      </div>
    </div>
  )
}
