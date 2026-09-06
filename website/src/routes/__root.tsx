import { Link, Outlet } from 'react-router-dom'
import { BeeLogo } from '../components/Logo'
import '../global.css'
import { LangProvider, useLang } from '../lib/i18n'
import { ThemeProvider, useTheme } from '../lib/theme'
import { Github, Globe, Monitor, Moon, Sparkles, Sun } from 'lucide-react'

function RootLayoutInner() {
  const { copy, lang, toggle } = useLang()
  const { theme, toggleNext } = useTheme()

  return (
    <div className="min-h-screen flex flex-col bg-[var(--bg-page)] text-[var(--text-primary)] font-sans selection:bg-amber-500/20 selection:text-amber-500 dark:selection:text-amber-200 transition-colors duration-200">
      {/* Background Ambient Lights */}
      <div className="ambient-glow top-[-100px] left-1/2 -translate-x-1/2 w-[600px] h-[350px] bg-amber-500/10 dark:bg-amber-500/15" />
      <div className="ambient-glow top-[400px] right-[-150px] w-[500px] h-[400px] bg-indigo-500/5 dark:bg-indigo-500/10" />

      {/* Floating Header */}
      <header className="sticky top-4 z-50 max-w-6xl mx-auto w-[calc(100%-2rem)] my-2">
        <nav className="glass-panel rounded-full px-5 py-3 flex items-center justify-between transition-all duration-300">
          <Link to="/" className="flex items-center gap-3 group">
            <BeeLogo className="w-7 h-7 transition-transform group-hover:scale-105" />
            <span className="text-lg font-bold tracking-tight text-zinc-950 dark:text-white font-display">
              BEEJS
            </span>
            <span className="hidden sm:inline-flex items-center gap-1 text-[10px] font-mono px-2 py-0.5 rounded-full bg-amber-500/10 text-amber-600 dark:text-amber-400 border border-amber-500/20 shadow-sm shadow-amber-500/10">
              <Sparkles className="w-2.5 h-2.5" /> v0.4.3
            </span>
          </Link>

          <div className="hidden md:flex items-center gap-8 text-sm font-medium text-zinc-600 dark:text-zinc-400">
            <Link to="/" className="hover:text-zinc-950 dark:hover:text-white transition-colors">
              {copy.nav.home}
            </Link>
            <Link to="/docs" className="hover:text-zinc-950 dark:hover:text-white transition-colors">
              {copy.nav.docs}
            </Link>
            <Link to="/blog" className="hover:text-zinc-950 dark:hover:text-white transition-colors">
              {copy.nav.blog}
            </Link>
          </div>

          <div className="flex items-center gap-2 sm:gap-3">
            {/* Theme Toggle Button */}
            <button
              type="button"
              onClick={toggleNext}
              className="flex items-center gap-1.5 px-3 py-1.5 rounded-full text-xs font-mono text-zinc-600 dark:text-zinc-400 hover:text-zinc-950 dark:hover:text-white hover:bg-zinc-200/60 dark:hover:bg-zinc-800/50 border border-transparent hover:border-zinc-300 dark:hover:border-zinc-700/50 transition-all cursor-pointer"
              title={copy.theme.toggle}
              aria-label={copy.theme.toggle}
            >
              {theme === 'system' && <Monitor className="w-3.5 h-3.5 text-amber-500 shrink-0" />}
              {theme === 'light' && <Sun className="w-3.5 h-3.5 text-amber-500 shrink-0" />}
              {theme === 'dark' && <Moon className="w-3.5 h-3.5 text-amber-400 shrink-0" />}
              <span className="hidden sm:inline text-[11px] font-medium">
                {theme === 'system' ? copy.theme.system : theme === 'light' ? copy.theme.light : copy.theme.dark}
              </span>
            </button>

            {/* Language Toggle */}
            <button
              type="button"
              onClick={toggle}
              className="flex items-center gap-1.5 px-3 py-1.5 rounded-full text-xs font-mono text-zinc-600 dark:text-zinc-400 hover:text-zinc-950 dark:hover:text-white hover:bg-zinc-200/60 dark:hover:bg-zinc-800/50 border border-transparent hover:border-zinc-300 dark:hover:border-zinc-700/50 transition-all cursor-pointer"
              aria-label={copy.toggle.label}
            >
              <Globe className="w-3.5 h-3.5 text-zinc-500 dark:text-zinc-400" />
              <span className={lang === 'en' ? 'text-amber-600 dark:text-amber-400 font-semibold' : ''}>{copy.toggle.en}</span>
              <span className="text-zinc-400 dark:text-zinc-600">/</span>
              <span className={lang === 'zh' ? 'text-amber-600 dark:text-amber-400 font-semibold' : ''}>{copy.toggle.zh}</span>
            </button>

            <a
              href="https://github.com/zh30/beejs"
              target="_blank"
              rel="noreferrer"
              className="flex items-center gap-2 px-3.5 sm:px-4 py-1.5 rounded-full bg-zinc-900 text-white dark:bg-white dark:text-zinc-950 font-semibold text-xs hover:bg-zinc-800 dark:hover:bg-zinc-200 transition-all shadow-sm"
            >
              <Github className="w-4 h-4" />
              <span className="hidden sm:inline">{copy.nav.github}</span>
            </a>
          </div>
        </nav>
      </header>

      <main className="grow relative z-10">
        <Outlet />
      </main>

      {/* Sleek Minimalist Footer */}
      <footer className="relative z-10 border-t border-zinc-200/80 dark:border-zinc-800/50 bg-white/70 dark:bg-[#07080a]/80 backdrop-blur-md mt-20">
        <div className="max-w-6xl mx-auto px-6 py-12 flex flex-col md:flex-row items-center justify-between gap-6 text-xs text-zinc-600 dark:text-zinc-500">
          <div className="flex items-center gap-3">
            <BeeLogo className="w-5 h-5 opacity-80" />
            <span className="text-zinc-800 dark:text-zinc-300 font-medium font-display">Beejs Runtime</span>
            <span className="text-zinc-400 dark:text-zinc-700">•</span>
            <span>{copy.footer.builtWith}</span>
          </div>

          <div className="flex items-center gap-6">
            <Link to="/docs" className="hover:text-zinc-950 dark:hover:text-zinc-300 transition-colors">
              {copy.footer.docs}
            </Link>
            <Link to="/blog" className="hover:text-zinc-950 dark:hover:text-zinc-300 transition-colors">
              {copy.footer.blog}
            </Link>
            <a
              href="https://github.com/zh30/beejs"
              target="_blank"
              rel="noreferrer"
              className="hover:text-zinc-950 dark:hover:text-zinc-300 transition-colors"
            >
              {copy.footer.githubRepo}
            </a>
          </div>

          <div className="text-zinc-500 dark:text-zinc-600">
            {copy.footer.copyright}
          </div>
        </div>
      </footer>
    </div>
  )
}

export default function RootLayout() {
  return (
    <ThemeProvider>
      <LangProvider>
        <RootLayoutInner />
      </LangProvider>
    </ThemeProvider>
  )
}
