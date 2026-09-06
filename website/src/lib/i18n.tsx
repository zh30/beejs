import React, { createContext, useContext, useEffect, useMemo, useState } from 'react'
import { en } from './locales/en'
import { zh } from './locales/zh'
import { es } from './locales/es'
import { fr } from './locales/fr'
import { hi } from './locales/hi'
import type { Lang, LangOption, TranslationSchema } from './locales/types'

export type { Lang, LangOption, TranslationSchema }

export const SUPPORTED_LANGS: readonly LangOption[] = [
  { code: 'en', label: 'English', nativeLabel: 'English', flag: '🇬🇧' },
  { code: 'zh', label: 'Chinese', nativeLabel: '简体中文', flag: '🇨🇳' },
  { code: 'es', label: 'Spanish', nativeLabel: 'Español', flag: '🇪🇸' },
  { code: 'fr', label: 'French', nativeLabel: 'Français', flag: '🇫🇷' },
  { code: 'hi', label: 'Hindi', nativeLabel: 'हिन्दी', flag: '🇮🇳' },
] as const

const copy: Record<Lang, TranslationSchema> = {
  en,
  zh,
  es,
  fr,
  hi,
}

export type LangContextValue = {
  lang: Lang
  setLang: (lang: Lang) => void
  toggle: () => void
  copy: TranslationSchema
  languages: readonly LangOption[]
}

export function resolveInitialLanguage(): Lang {
  if (typeof window === 'undefined') return 'en'

  // 1. Check window.__BEEJS_INITIAL_LANG__ set by index.html pre-hydration script
  const windowLang = (window as unknown as { __BEEJS_INITIAL_LANG__?: string }).__BEEJS_INITIAL_LANG__
  if (
    windowLang &&
    (windowLang === 'en' ||
      windowLang === 'zh' ||
      windowLang === 'es' ||
      windowLang === 'fr' ||
      windowLang === 'hi')
  ) {
    return windowLang as Lang
  }

  // 2. Check localStorage
  try {
    const stored = window.localStorage.getItem('beejs_lang')
    if (
      stored &&
      (stored === 'en' ||
        stored === 'zh' ||
        stored === 'es' ||
        stored === 'fr' ||
        stored === 'hi')
    ) {
      return stored as Lang
    }
  } catch (e) {}

  // 3. Detect browser languages via navigator.languages or navigator.language
  try {
    const navLangs = window.navigator.languages || [window.navigator.language || '']
    const supportedCodes: Lang[] = ['en', 'zh', 'hi', 'es', 'fr']
    for (const bLang of navLangs) {
      if (!bLang) continue
      const prefix = bLang.toLowerCase().split('-')[0] as Lang
      if (supportedCodes.includes(prefix)) {
        return prefix
      }
    }
  } catch (e) {}

  // 4. Default fallback: English
  return 'en'
}

const LangContext = createContext<LangContextValue | null>(null)

export function LangProvider({ children }: { children: React.ReactNode }) {
  const [lang, setLangState] = useState<Lang>(resolveInitialLanguage)

  const setLang = (nextLang: Lang) => {
    setLangState(nextLang)
    try {
      window.localStorage.setItem('beejs_lang', nextLang)
    } catch (e) {}
    if (typeof document !== 'undefined') {
      document.documentElement.lang = nextLang
    }
  }

  useEffect(() => {
    if (typeof document !== 'undefined') {
      document.documentElement.lang = lang
    }
  }, [lang])

  const value = useMemo<LangContextValue>(() => {
    const nextIdx =
      (SUPPORTED_LANGS.findIndex((l) => l.code === lang) + 1) % SUPPORTED_LANGS.length
    return {
      lang,
      setLang,
      toggle: () => setLang(SUPPORTED_LANGS[nextIdx].code),
      copy: copy[lang],
      languages: SUPPORTED_LANGS,
    }
  }, [lang])

  return <LangContext.Provider value={value}>{children}</LangContext.Provider>
}

export function useLang() {
  const ctx = useContext(LangContext)
  if (!ctx) {
    throw new Error('useLang must be used within LangProvider')
  }
  return ctx
}
