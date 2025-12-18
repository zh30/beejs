import { Link, Outlet } from 'react-router-dom'
import { BeeLogo } from '../components/Logo'
import '../global.css'

export default function RootLayout() {
  return (
    <div className="min-h-screen flex flex-col bg-brand-black text-gray-100 font-sans antialiased">
      <header className="sticky top-0 z-50 glass border-b border-white/10 px-4 py-3 md:px-8">
        <nav className="max-w-7xl mx-auto flex items-center justify-between">
          <Link to="/" className="flex items-center space-x-3 group">
            <BeeLogo className="w-8 h-8 group-hover:scale-110 transition-transform" />
            <span className="text-2xl font-black bg-linear-to-r from-brand-yellow to-yellow-500 bg-clip-text text-transparent uppercase italic tracking-tighter">
              Beejs
            </span>
          </Link>

          <div className="hidden md:flex items-center space-x-8 text-xs font-bold uppercase tracking-widest px-6">
            <Link to="/" className="hover:text-brand-yellow transition-colors relative group">
              Home
              <span className="absolute -bottom-1 left-0 w-full h-px bg-brand-yellow scale-x-0 transition-transform group-hover:scale-x-100" />
            </Link>
            <Link to="/docs" className="hover:text-brand-yellow transition-colors relative group">
              Docs
              <span className="absolute -bottom-1 left-0 w-full h-px bg-brand-yellow scale-x-0 transition-transform group-hover:scale-x-100" />
            </Link>
            <Link to="/blog" className="hover:text-brand-yellow transition-colors relative group">
              Blog
              <span className="absolute -bottom-1 left-0 w-full h-px bg-brand-yellow scale-x-0 transition-transform group-hover:scale-x-100" />
            </Link>
          </div>

          <div className="flex items-center space-x-4">
            <a
              href="https://github.com/zh30/beejs"
              target="_blank"
              rel="noreferrer"
              className="px-6 py-2 bg-brand-yellow text-brand-black font-black text-sm hover:bg-white transition-all -skew-x-12 uppercase"
            >
              <span className="inline-block skew-x-12">GitHub</span>
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
            <h4 className="font-bold mb-4 text-white uppercase tracking-widest text-xs">Contact</h4>
            <p className="text-sm text-gray-500 font-mono">
              Email: support@beejs.zhanghe.dev
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
