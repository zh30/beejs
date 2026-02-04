import { Link, Outlet } from 'react-router-dom'
import { BeeLogo } from '../components/Logo'
import '../global.css'
import { LangProvider, useLang } from '../lib/i18n'

function RootLayoutInner() {
  const { copy, lang, toggle } = useLang()

  return (
    <div className="min-h-screen flex flex-col bg-hud-void text-hud-text font-sans antialiased">
      <header className="sticky top-0 z-50 border-b border-hud-line/60 bg-hud-void/90 backdrop-blur">
        <nav className="max-w-7xl mx-auto flex items-center justify-between px-4 py-4 md:px-8">
          <Link to="/" className="flex items-center gap-3">
            <BeeLogo className="w-8 h-8" />
            <span className="text-2xl font-display tracking-[0.2em] uppercase text-hud-text">
              BEEJS
            </span>
          </Link>

          <div className="hidden md:flex items-center gap-10 text-[11px] uppercase tracking-[0.4em] text-hud-muted">
            <Link to="/" className="hover:text-hud-text transition-colors">
              {copy.nav.home}
            </Link>
            <Link to="/docs" className="hover:text-hud-text transition-colors">
              {copy.nav.docs}
            </Link>
            <Link to="/blog" className="hover:text-hud-text transition-colors">
              {copy.nav.blog}
            </Link>
          </div>

          <div className="flex items-center gap-4">
            <button
              type="button"
              onClick={toggle}
              className="flex items-center gap-2 text-[10px] uppercase tracking-[0.4em] text-hud-muted hover:text-hud-text transition-colors"
              aria-label={copy.toggle.label}
            >
              <span className={lang === 'en' ? 'text-hud-accent' : ''}>{copy.toggle.en}</span>
              <span className="opacity-40">/</span>
              <span className={lang === 'zh' ? 'text-hud-accent' : ''}>{copy.toggle.zh}</span>
            </button>
            <a
              href="https://github.com/zh30/beejs"
              target="_blank"
              rel="noreferrer"
              className="hud-button hud-button-primary"
            >
              {copy.nav.github}
            </a>
          </div>
        </nav>
        <div className="md:hidden border-t border-hud-line/60 px-4 py-2 flex items-center justify-between text-[10px] uppercase tracking-[0.35em] text-hud-muted">
          <Link to="/" className="hover:text-hud-text transition-colors">
            {copy.nav.home}
          </Link>
          <Link to="/docs" className="hover:text-hud-text transition-colors">
            {copy.nav.docs}
          </Link>
          <Link to="/blog" className="hover:text-hud-text transition-colors">
            {copy.nav.blog}
          </Link>
        </div>
      </header>

      <main className="grow">
        <Outlet />
      </main>

      <footer className="border-t border-hud-line/60 bg-hud-deep/70">
        <div className="max-w-7xl mx-auto px-4 md:px-8 py-8 flex flex-col md:flex-row items-start md:items-center justify-between gap-6 text-[11px] uppercase tracking-[0.4em] text-hud-muted">
          <div className="flex flex-col gap-2">
            <span>{copy.footer.statusLabel}</span>
            <span className="text-hud-text">{copy.footer.statusValue}</span>
          </div>
          <div className="flex flex-col gap-2">
            <span>{copy.footer.stage}</span>
            <span className="text-hud-text">Rust + V8</span>
          </div>
          <div className="flex flex-col gap-2">
            <span>{copy.footer.contact}</span>
            <span className="text-hud-text normal-case tracking-[0.2em]">{copy.footer.email}</span>
          </div>
        </div>
        <div className="border-t border-hud-line/40 text-center text-[10px] uppercase tracking-[0.35em] text-hud-muted py-4">
          {new Date().getFullYear()} Beejs. {copy.footer.rights}
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
