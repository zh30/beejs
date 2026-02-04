import { motion } from 'framer-motion'
import type { ReactNode } from 'react'
import { Activity, Cpu, Gauge, Layers, Rocket, Server } from 'lucide-react'
import { Link } from 'react-router-dom'
import { BeeLogo } from '../components/Logo'
import { useLang } from '../lib/i18n'

const systemIcons = [Cpu, Rocket, Server, Activity, Layers, Gauge]

export default function HomeComponent() {
  const { copy } = useLang()
  const home = copy.home

  return (
    <div className="relative overflow-hidden bg-hud-void">
      <div className="absolute inset-0 hud-grid opacity-40 pointer-events-none" />
      <div className="scanline" />
      <div className="absolute top-[-20%] right-[-10%] w-80 h-80 bg-hud-accent/10 blur-[120px]" />

      <section className="relative max-w-7xl mx-auto px-4 md:px-8 pt-24 pb-16">
        <motion.div
          initial={{ opacity: 0, y: 24 }}
          animate={{ opacity: 1, y: 0 }}
          transition={{ duration: 0.6 }}
          className="hud-panel relative p-8 md:p-14"
        >
          <span className="hud-corner corner-tl" />
          <span className="hud-corner corner-tr" />
          <span className="hud-corner corner-bl" />
          <span className="hud-corner corner-br" />

          <div className="flex items-center justify-between">
            <span className="hud-tag">{home.kicker}</span>
            <span className="text-[10px] uppercase tracking-[0.4em] text-hud-muted">SYS-93</span>
          </div>

          <div className="grid grid-cols-1 lg:grid-cols-[1.2fr_0.8fr] gap-10 mt-10">
            <div>
              <div className="flex items-center gap-4 mb-6">
                <BeeLogo className="w-12 h-12" />
                <span className="text-[11px] uppercase tracking-[0.5em] text-hud-muted">Rust + V8</span>
              </div>
              <h1 className="text-4xl md:text-6xl font-display uppercase tracking-[0.2em]">
                {home.title}{' '}
                <span className="text-hud-accent hud-glow">{home.titleAccent}</span>
              </h1>
              <p className="mt-6 text-hud-muted text-sm md:text-base max-w-xl leading-relaxed">
                {home.subtitle}
              </p>
              <div className="mt-8 flex flex-wrap gap-4">
                <Link to="/docs" className="hud-button hud-button-primary">
                  {home.ctaPrimary}
                </Link>
                <Link to="/blog" className="hud-button">
                  {home.ctaSecondary}
                </Link>
              </div>
            </div>

            <div className="hud-panel-soft p-8 relative">
              <div className="flex items-center justify-between">
                <span className="hud-tag">{home.heroMetricLabel}</span>
                <span className="text-[10px] uppercase tracking-[0.4em] text-hud-muted">CORE</span>
              </div>
              <div className="mt-8">
                <div className="text-5xl md:text-6xl font-display text-hud-text">
                  {home.heroMetricValue}
                </div>
                <div className="text-sm uppercase tracking-[0.4em] text-hud-accent mt-2">
                  {home.heroMetricUnit}
                </div>
                <p className="text-xs text-hud-muted mt-4">{home.heroMetricNote}</p>
              </div>
              <div className="mt-10 h-px w-full bg-hud-line/70" />
              <div className="mt-6 text-[10px] uppercase tracking-[0.35em] text-hud-muted">
                {home.heroFootnote}
              </div>
            </div>
          </div>
        </motion.div>
      </section>

      <section className="relative max-w-7xl mx-auto px-4 md:px-8 pb-20">
        <div className="flex flex-col md:flex-row md:items-end justify-between gap-8 mb-10">
          <div>
            <p className="hud-tag">{home.telemetryTitle}</p>
            <h2 className="text-3xl md:text-4xl font-display uppercase tracking-[0.18em] mt-4">
              {home.telemetrySubtitle}
            </h2>
          </div>
          <p className="text-xs text-hud-muted max-w-sm">{home.telemetryNote}</p>
        </div>
        <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-4">
          {home.telemetry.map((item) => (
            <TelemetryCard key={item.label} {...item} />
          ))}
        </div>
      </section>

      <section className="relative max-w-7xl mx-auto px-4 md:px-8 pb-24">
        <div className="hud-panel-soft p-8 md:p-12">
          <div className="flex flex-col md:flex-row md:items-end justify-between gap-8">
            <div>
              <p className="hud-tag">{home.benchmarksTitle}</p>
              <h3 className="text-3xl md:text-4xl font-display uppercase tracking-[0.18em] mt-4">
                {home.benchmarksSubtitle}
              </h3>
            </div>
            <div className="text-[10px] uppercase tracking-[0.35em] text-hud-muted">
              {home.benchmarksMeta}
            </div>
          </div>
          <div className="grid grid-cols-1 md:grid-cols-3 gap-6 mt-10">
            {home.benchmarks.map((bench) => (
              <BenchmarkCard key={bench.label} {...bench} />
            ))}
          </div>
        </div>
      </section>

      <section className="relative max-w-7xl mx-auto px-4 md:px-8 pb-24">
        <div className="flex flex-col md:flex-row md:items-end justify-between gap-8 mb-10">
          <div>
            <p className="hud-tag">{home.systemsTitle}</p>
            <h3 className="text-3xl md:text-4xl font-display uppercase tracking-[0.18em] mt-4">
              {home.systemsSubtitle}
            </h3>
          </div>
          <div className="text-[10px] uppercase tracking-[0.35em] text-hud-muted">
            {home.systemsMeta}
          </div>
        </div>
        <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
          {home.systems.map((system, index) => {
            const Icon = systemIcons[index % systemIcons.length]
            return (
              <SystemCard
                key={system.title}
                title={system.title}
                desc={system.desc}
                icon={<Icon className="w-5 h-5 text-hud-accent" />}
              />
            )
          })}
        </div>
      </section>

      <section className="relative max-w-7xl mx-auto px-4 md:px-8 pb-32">
        <div className="hud-panel p-10 md:p-14 text-center relative">
          <span className="hud-corner corner-tl" />
          <span className="hud-corner corner-tr" />
          <span className="hud-corner corner-bl" />
          <span className="hud-corner corner-br" />
          <h3 className="text-3xl md:text-5xl font-display uppercase tracking-[0.2em]">
            {home.ctaTitle}
          </h3>
          <p className="text-hud-muted mt-4 max-w-2xl mx-auto">
            {home.ctaSubtitle}
          </p>
          <div className="mt-8 flex justify-center">
            <Link to="/docs" className="hud-button hud-button-primary">
              {home.ctaButton}
            </Link>
          </div>
        </div>
      </section>
    </div>
  )
}

function TelemetryCard({ label, value, delta, note }: { label: string; value: string; delta: string; note: string }) {
  return (
    <div className="hud-panel-soft p-6 relative">
      <span className="hud-tag">{label}</span>
      <div className="mt-6 text-2xl font-display text-hud-text">{value}</div>
      <div className="mt-3 flex items-center justify-between text-[10px] uppercase tracking-[0.3em] text-hud-muted">
        <span>{note}</span>
        <span className="text-hud-accent">{delta}</span>
      </div>
    </div>
  )
}

function BenchmarkCard({ label, value, unit, delta }: { label: string; value: string; unit: string; delta: string }) {
  return (
    <div className="hud-panel p-6">
      <div className="text-[10px] uppercase tracking-[0.35em] text-hud-muted">{label}</div>
      <div className="mt-6 text-3xl font-display text-hud-text">{value}</div>
      <div className="text-xs uppercase tracking-[0.3em] text-hud-accent mt-2">{unit}</div>
      <div className="mt-6 text-[10px] uppercase tracking-[0.3em] text-hud-muted">{delta}</div>
    </div>
  )
}

function SystemCard({ title, desc, icon }: { title: string; desc: string; icon: ReactNode }) {
  return (
    <div className="hud-panel-soft p-6 flex flex-col gap-4">
      <div className="flex items-center justify-between">
        {icon}
        <span className="text-[10px] uppercase tracking-[0.35em] text-hud-muted">
          {home.systemsLabel}
        </span>
      </div>
      <h4 className="text-xl font-display uppercase tracking-[0.18em]">{title}</h4>
      <p className="text-sm text-hud-muted leading-relaxed">{desc}</p>
    </div>
  )
}
