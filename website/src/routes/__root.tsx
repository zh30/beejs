import { Link, Outlet } from 'react-router-dom'
import '../global.css'

export default function RootLayout() {
  return (
    <div className="min-h-screen flex flex-col bg-brand-black text-gray-100 font-sans antialiased">
      <header className="sticky top-0 z-50 glass border-b border-white/10 px-4 py-3 md:px-8">
        <nav className="max-w-7xl mx-auto flex items-center justify-between">
          <Link to="/" className="flex items-center space-x-2">
            <span className="text-2xl font-bold bg-linear-to-r from-brand-yellow to-yellow-500 bg-clip-text text-transparent">
              Beejs
            </span>
          </Link>

          <div className="hidden md:flex items-center space-x-8 text-sm font-medium">
            <Link to="/" className="hover:text-brand-yellow transition-colors">
              Home
            </Link>
            <Link to="/docs" className="hover:text-brand-yellow transition-colors">
              Docs
            </Link>
            <Link to="/blog" className="hover:text-brand-yellow transition-colors">
              Blog
            </Link>
          </div>

          <div className="flex items-center space-x-4">
            <a
              href="https://github.com/zh30/beejs"
              target="_blank"
              rel="noreferrer"
              className="px-4 py-2 bg-brand-yellow text-brand-black font-bold rounded-lg hover:bg-yellow-400 transition-all text-sm md:text-base"
            >
              GitHub
            </a>
          </div>
        </nav>
      </header>

      <main className="grow">
        <Outlet />
      </main>

      <footer className="bg-brand-gray/50 border-t border-white/5 py-12 px-4">
        <div className="max-w-7xl mx-auto grid grid-cols-1 md:grid-cols-3 gap-8 text-center md:text-left">
          <div>
            <h3 className="text-lg font-bold mb-4 text-white">Beejs</h3>
            <p className="text-gray-400 text-sm">
              Built for speed. Optimized for AI. <br />
              The future of JavaScript runtimes.
            </p>
          </div>
          <div>
            <h4 className="font-bold mb-4 text-white">Links</h4>
            <ul className="space-y-2 text-sm text-gray-400">
              <li><Link to="/docs" className="hover:text-white">Documentation</Link></li>
              <li><Link to="/blog" className="hover:text-white">Blog</Link></li>
              <li><a href="https://github.com/zh30/beejs" className="hover:text-white">GitHub</a></li>
            </ul>
          </div>
          <div>
            <h4 className="font-bold mb-4 text-white">Contact</h4>
            <p className="text-sm text-gray-400">
              Email: support@beejs.zhanghe.dev <br />
              Discord: discord.gg/beejs
            </p>
          </div>
        </div>
        <div className="max-w-7xl mx-auto mt-12 pt-8 border-t border-white/5 text-center text-xs text-gray-500">
          &copy; {new Date().getFullYear()} Beejs. All rights reserved.
        </div>
      </footer>
    </div>
  )
}
