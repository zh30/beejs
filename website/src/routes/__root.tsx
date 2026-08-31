import { Link, Outlet } from 'react-router-dom'
import { BeeLogo } from '../components/Logo'
import '../global.css'
import { LangProvider, useLang } from '../lib/i18n'
import { Github, Globe, Sparkles } from 'lucide-react'

function RootLayoutInner() {
  const { copy, lang, toggle } = useLang()

  return (
    <div className="min-h-screen flex flex-col bg-[#07080a] text-zinc-100 font-sans selection:bg-amber-500/20 selection:text-amber-200">
      {/* Background Ambient Lights */}
      <div className="ambient-glow top-[-100px] left-1/2 -translate-x-1/2 w-[600px] h-[350px] bg-amber-500/10" />
      <div className="ambient-glow top-[400px] right-[-150px] w-[500px] h-[400px] bg-indigo-500/5" />

      {/* Floating Header */}
      <header className="sticky top-4 z-50 max-w-6xl mx-auto w-[calc(100%-2rem)] my-2">
        <nav className="glass-panel rounded-full px-5 py-3 flex items-center justify-between transition-all duration-300">
          <Link to="/" className="flex items-center gap-3 group">
            <BeeLogo className="w-7 h-7 transition-transform group-hover:scale-105" />
            <span className="text-lg font-bold tracking-tight text-white font-display">
              BEEJS
            </span>
            <span className="hidden sm:inline-flex items-center gap-1 text-[10px] font-mono px-2 py-0.5 rounded-full bg-amber-500/10 text-amber-400 border border-amber-500/20">
              <Sparkles className="w-2.5 h-2.5" /> v0.1.1
            </span>
          </Link>

          <div className="hidden md:flex items-center gap-8 text-sm font-medium text-zinc-400">
            <Link to="/" className="hover:text-white transition-colors">
              {copy.nav.home}
            </Link>
            <Link to="/docs" className="hover:text-white transition-colors">
              {copy.nav.docs}
            </Link>
            <Link to="/blog" className="hover:text-white transition-colors">
              {copy.nav.blog}
            </Link>
          </div>

          <div className="flex items-center gap-3">
            <button
              type="button"
              onClick={toggle}
              className="flex items-center gap-1.5 px-3 py-1.5 rounded-full text-xs font-mono text-zinc-400 hover:text-white hover:bg-zinc-800/50 border border-transparent hover:border-zinc-700/50 transition-all"
              aria-label={copy.toggle.label}
            >
              <Globe className="w-3.5 h-3.5 text-zinc-400" />
              <span className={lang === 'en' ? 'text-amber-400 font-semibold' : ''}>{copy.toggle.en}</span>
              <span className="text-zinc-600">/</span>
              <span className={lang === 'zh' ? 'text-amber-400 font-semibold' : ''}>{copy.toggle.zh}</span>
            </button>

            <a
              href="https://github.com/zh30/beejs"
              target="_blank"
              rel="noreferrer"
              className="flex items-center gap-2 px-4 py-1.5 rounded-full bg-white text-zinc-950 font-semibold text-xs hover:bg-zinc-200 transition-all shadow-sm"
            >
              <Github className="w-4 h-4" />
              <span>{copy.nav.github}</span>
            </a>
          </div>
        </nav>
      </header>

      <main className="grow relative z-10">
        <Outlet />
      </main>

      {/* Sleek Minimalist Footer */}
      <footer className="relative z-10 border-t border-zinc-800/50 bg-[#07080a]/80 backdrop-blur-md mt-20">
        <div className="max-w-6xl mx-auto px-6 py-12 flex flex-col md:flex-row items-center justify-between gap-6 text-xs text-zinc-500">
          <div className="flex items-center gap-3">
            <BeeLogo className="w-5 h-5 opacity-80" />
            <span className="text-zinc-300 font-medium font-display">Beejs Runtime</span>
            <span className="text-zinc-700">•</span>
            <span>{copy.footer.builtWith}</span>
          </div>

          <div className="flex items-center gap-6">
            <Link to="/docs" className="hover:text-zinc-300 transition-colors">
              {copy.footer.docs}
            </Link>
            <Link to="/blog" className="hover:text-zinc-300 transition-colors">
              {copy.footer.blog}
            </Link>
            <a
              href="https://github.com/zh30/beejs"
              target="_blank"
              rel="noreferrer"
              className="hover:text-zinc-300 transition-colors"
            >
              {copy.footer.githubRepo}
            </a>
          </div>

          <div className="text-zinc-600">
            {copy.footer.copyright}
          </div>
        </div>
      </footer>
    </div>
  )
}

export default function RootLayout() {
  return (
    <LangProvider>
      <RootLayoutInner />
    </LangProvider>
  )
}
