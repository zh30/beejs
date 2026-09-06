import React, { createContext, useContext, useEffect, useState } from "react"

export type Theme = "system" | "dark" | "light"
export type ResolvedTheme = "dark" | "light"

interface ThemeContextValue {
  theme: Theme
  resolvedTheme: ResolvedTheme
  setTheme: (theme: Theme) => void
  toggleNext: () => void
}

const STORAGE_KEY = "beejs-theme"

const ThemeContext = createContext<ThemeContextValue | null>(null)

function getSystemTheme(): ResolvedTheme {
  if (typeof window === "undefined") return "dark"
  return window.matchMedia("(prefers-color-scheme: dark)").matches ? "dark" : "light"
}

function getStoredTheme(): Theme {
  if (typeof window === "undefined") return "system"
  try {
    const saved = localStorage.getItem(STORAGE_KEY)
    if (saved === "dark" || saved === "light" || saved === "system") {
      return saved
    }
  } catch (e) {}
  return "system"
}

export function ThemeProvider({ children }: { children: React.ReactNode }) {
  const [theme, setThemeState] = useState<Theme>(getStoredTheme)
  const [resolvedTheme, setResolvedTheme] = useState<ResolvedTheme>(() => {
    const stored = getStoredTheme()
    return stored === "system" ? getSystemTheme() : stored
  })

  const applyTheme = (resolved: ResolvedTheme) => {
    if (typeof document === "undefined") return
    const root = document.documentElement
    if (resolved === "dark") {
      root.classList.add("dark")
      root.style.colorScheme = "dark"
    } else {
      root.classList.remove("dark")
      root.style.colorScheme = "light"
    }
  }

  const setTheme = (newTheme: Theme) => {
    setThemeState(newTheme)
    try {
      localStorage.setItem(STORAGE_KEY, newTheme)
    } catch (e) {}

    const nextResolved = newTheme === "system" ? getSystemTheme() : newTheme
    setResolvedTheme(nextResolved)
    applyTheme(nextResolved)
  }

  const toggleNext = () => {
    // Cycles: system -> light -> dark -> system
    if (theme === "system") {
      setTheme("light")
    } else if (theme === "light") {
      setTheme("dark")
    } else {
      setTheme("system")
    }
  }

  useEffect(() => {
    const mediaQuery = window.matchMedia("(prefers-color-scheme: dark)")
    const handleChange = (e: MediaQueryListEvent) => {
      if (theme === "system") {
        const nextResolved: ResolvedTheme = e.matches ? "dark" : "light"
        setResolvedTheme(nextResolved)
        applyTheme(nextResolved)
      }
    }

    mediaQuery.addEventListener("change", handleChange)
    return () => mediaQuery.removeEventListener("change", handleChange)
  }, [theme])

  useEffect(() => {
    const currentResolved = theme === "system" ? getSystemTheme() : theme
    setResolvedTheme(currentResolved)
    applyTheme(currentResolved)
  }, [theme])

  return (
    <ThemeContext.Provider value={{ theme, resolvedTheme, setTheme, toggleNext }}>
      {children}
    </ThemeContext.Provider>
  )
}

export function useTheme() {
  const context = useContext(ThemeContext)
  if (!context) {
    throw new Error("useTheme must be used within a ThemeProvider")
  }
  return context
}
