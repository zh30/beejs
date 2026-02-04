# Bridge HUD Website Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Rebuild the `/website` UI into a minimalist sci‑fi “Bridge HUD” design with bilingual (EN/中文) toggle and performance‑forward homepage.

**Architecture:** Use a small set of reusable UI patterns (HUD panels, metric rails, system cards) applied across Home/Docs/Blog. Add a lightweight language dictionary and toggle in the root layout to drive bilingual strings without a heavy i18n library. Update global styles to define the new visual system and animations.

**Tech Stack:** React + TypeScript + Vite, Tailwind v4 (via `@theme` in CSS), lucide-react, framer-motion.

---

### Task 1: Establish the global HUD theme

**Files:**
- Modify: `website/src/global.css`

**Step 1: Update theme tokens and fonts**
- Replace existing brand colors with navy/steel/cyan palette.
- Define font stack for display/body/mono (non‑generic fonts).

**Step 2: Add HUD utilities**
- Add classes for grid background, panel borders, corner marks, and scanlines.

**Step 3: Visual check**
Run: `pnpm -C website dev`
Expected: app starts without CSS errors; background grid and panel styles available.

**Step 4: Commit**
```bash
git add website/src/global.css
git commit -m "feat(website): establish bridge HUD theme"
```

### Task 2: Add bilingual dictionary + language toggle

**Files:**
- Create: `website/src/lib/i18n.ts`
- Modify: `website/src/routes/__root.tsx`

**Step 1: Create bilingual strings dictionary**
- Define EN/中文 content for nav, footer, and shared labels.

**Step 2: Wire toggle in root layout**
- Add language state and a small toggle in the header.
- Pass language down via context or props.

**Step 3: Visual check**
Run: `pnpm -C website dev`
Expected: clicking toggle switches labels between EN/中文.

**Step 4: Commit**
```bash
git add website/src/lib/i18n.ts website/src/routes/__root.tsx
git commit -m "feat(website): add bilingual UI toggle"
```

### Task 3: Rebuild Home (performance‑first)

**Files:**
- Modify: `website/src/routes/index.tsx`

**Step 1: Replace hero with Bridge Console layout**
- Large focal metric + concise bilingual subhead.
- Minimal CTA buttons aligned with HUD styling.

**Step 2: Rebuild telemetry + benchmarks sections**
- 3–4 metric modules with clean data hierarchy.
- Benchmark grid styled as instrumentation panels.

**Step 3: Visual check**
Run: `pnpm -C website dev`
Expected: hero, telemetry, and benchmarks align to the new HUD look.

**Step 4: Commit**
```bash
git add website/src/routes/index.tsx
git commit -m "feat(website): rebuild home bridge HUD"
```

### Task 4: Rebuild Docs as “Ship Manual”

**Files:**
- Modify: `website/src/routes/docs.tsx`

**Step 1: Add manual layout**
- Left nav list, right content panel.
- Section numbering and succinct bilingual copy.

**Step 2: Visual check**
Run: `pnpm -C website dev`
Expected: docs page reads as clean manual with HUD styling.

**Step 3: Commit**
```bash
git add website/src/routes/docs.tsx
git commit -m "feat(website): rebuild docs manual layout"
```

### Task 5: Rebuild Blog as “Flight Log”

**Files:**
- Modify: `website/src/routes/blog.tsx`

**Step 1: Add flight log timeline**
- Minimal timeline line, log cards, date stamps.

**Step 2: Visual check**
Run: `pnpm -C website dev`
Expected: blog shows a clean log timeline with HUD panels.

**Step 3: Commit**
```bash
git add website/src/routes/blog.tsx
git commit -m "feat(website): rebuild blog flight log"
```

### Task 6: Polish shared components and assets

**Files:**
- Modify: `website/src/components/Logo.tsx`
- Modify: `website/src/routes/__root.tsx`

**Step 1: Align logo + footer with new visual system**
- Ensure logo blends with dark HUD background.
- Footer becomes a compact system status row.

**Step 2: Visual check**
Run: `pnpm -C website dev`
Expected: nav/footer look cohesive with the rest of the site.

**Step 3: Commit**
```bash
git add website/src/components/Logo.tsx website/src/routes/__root.tsx
git commit -m "feat(website): polish shared chrome"
```

### Task 7: Final pass + review

**Files:**
- Modify: `website/src/routes/index.tsx`
- Modify: `website/src/routes/docs.tsx`
- Modify: `website/src/routes/blog.tsx`
- Modify: `website/src/global.css`

**Step 1: Consistency sweep**
- Check spacing, typography rhythm, and hover/focus states.

**Step 2: Manual verification**
Run: `pnpm -C website dev`
Expected: no console errors; layout responsive on mobile and desktop.

**Step 3: Commit**
```bash
git add website/src/routes/index.tsx website/src/routes/docs.tsx website/src/routes/blog.tsx website/src/global.css
git commit -m "chore(website): final visual polish"
```
