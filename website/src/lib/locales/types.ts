export type Lang = 'en' | 'zh' | 'es' | 'fr' | 'hi'

export interface LangOption {
  code: Lang
  label: string
  nativeLabel: string
  flag: string
}

export interface BenchmarkItem {
  id: string
  category: 'core' | 'io'
  title: string
  desc: string
  beeValue: string
  beeOps: string
  bunValue: string
  bunOps: string
  nodeValue: string
  nodeOps: string
  multiplier: string
  isBeeWinner: boolean
  beeBar: number
  bunBar: number
  nodeBar: number
}

export interface TelemetryItem {
  label: string
  value: string
  delta: string
  note: string
}

export interface FeatureItem {
  title: string
  desc: string
}

export interface SystemItem {
  title: string
  desc: string
}

export interface DocSection {
  title: string
  subtitle: string
  body?: readonly string[]
  cards?: readonly { title: string; desc: string }[]
  code?: readonly string[]
  list?: readonly string[]
}

export interface TranslationSchema {
  nav: {
    home: string
    docs: string
    blog: string
    github: string
  }
  toggle: {
    label: string
    en: string
    zh: string
    es: string
    fr: string
    hi: string
  }
  theme: {
    system: string
    light: string
    dark: string
    toggle: string
  }
  footer: {
    statusLabel: string
    statusValue: string
    stage: string
    contact: string
    email: string
    rights: string
    builtWith: string
    docs: string
    blog: string
    githubRepo: string
    copyright: string
  }
  home: {
    heroBadge: string
    heroBadgeSub: string
    heroBanner: string
    heroBannerLink: string
    heroTitlePrefix: string
    heroTitleAccent: string
    heroTitleSuffix: string
    heroSubtitle: string
    ctaPrimary: string
    ctaSecondary: string
    ctaNotes: string
    copyBtn: string
    copiedBtn: string
    latestArticle: {
      badge: string
      title: string
      desc: string
      readTime: string
      date: string
      link: string
      action: string
    }
    benchmarksHeader: string
    benchmarksSub: string
    benchmarksNote: string
    benchmarksFilterAll: string
    benchmarksFilterCore: string
    benchmarksFilterIo: string
    benchmarksFastest: string
    benchmarksParity: string
    benchmarks: BenchmarkItem[]
    telemetryTitle: string
    telemetrySubtitle: string
    telemetryNote: string
    telemetry: TelemetryItem[]
    sandboxTitle: string
    sandboxTag: string
    sandboxComment: string
    sandboxLog: string
    sandboxBoot: string
    featuresTitle: string
    featuresSubtitle: string
    features: FeatureItem[]
    systemsTitle: string
    systemsSubtitle: string
    systemsMeta: string
    systemsLabel: string
    systems: SystemItem[]
    ctaTitle: string
    ctaSubtitle: string
    ctaButton: string
    ctaNotesButton: string
  }
  docs: {
    title: string
    subtitle: string
    backToHome: string
    groups: readonly {
      title: string
      items: readonly { id: string; label: string }[]
    }[]
    sections: {
      introduction: DocSection
      installation: DocSection
      'quick-start': DocSection
      'v8-isolate-pool': DocSection
      'jit-optimization': DocSection
      'memory-management': DocSection
      'server-mode': DocSection
      'cli-usage': DocSection
      'api-reference': DocSection
      modules: DocSection
    }
  }
  blog: {
    title: string
    subtitle: string
    tagLabel: string
    back: string
    operator: string
    by: string
    timestamp: string
    readTime: string
    readMore: string
    notFound: string
    fallbackNote: string
  }
}
