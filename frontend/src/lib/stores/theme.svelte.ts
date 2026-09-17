import { type CustomPalette, getTerminalTheme, type ITheme } from '$lib/terminal/themes'

export type { CustomPalette }

export interface ThemeSettings {
  id: string
  name: string
  isDark: boolean
}

export const BUILTIN_THEMES: ThemeSettings[] = [
  { id: 'tokyo-night', name: 'Tokyo Night', isDark: true },
  { id: 'catppuccin-mocha', name: 'Catppuccin Mocha', isDark: true },
  { id: 'jetbrains-dark', name: 'JetBrains Dark', isDark: true },
  { id: 'one-dark', name: 'One Dark', isDark: true },
  { id: 'github-light', name: 'GitHub Light', isDark: false },
  { id: 'custom', name: 'Custom Palette', isDark: true },
]

export const DEFAULT_CUSTOM_PALETTE: CustomPalette = {
  bgPrimary: '#1e1f22',
  bgSecondary: '#2b2d30',
  bgTertiary: '#191a1c',
  textPrimary: '#dfe1e5',
  textSecondary: '#9da0a8',
  accent: '#3574f0',
  border: '#393b40',
}

let currentTheme = $state<string>('jetbrains-dark')
let customPalette = $state<CustomPalette>({ ...DEFAULT_CUSTOM_PALETTE })

export function getTheme(): string {
  return currentTheme
}

export function getCustomPalette(): CustomPalette {
  return customPalette
}

export function setCustomPalette(palette: Partial<CustomPalette>) {
  customPalette = { ...customPalette, ...palette }
  if (typeof localStorage !== 'undefined') {
    localStorage.setItem('slopssh-custom-palette', JSON.stringify(customPalette))
  }
  if (currentTheme === 'custom') {
    applyCustomPaletteToCss(customPalette)
  }
}

function applyCustomPaletteToCss(p: CustomPalette) {
  if (typeof document === 'undefined') return
  const root = document.documentElement
  root.style.setProperty('--bg-primary', p.bgPrimary)
  root.style.setProperty('--bg-secondary', p.bgSecondary)
  root.style.setProperty('--bg-tertiary', p.bgTertiary)
  root.style.setProperty('--bg-input', p.bgTertiary)
  root.style.setProperty('--text-primary', p.textPrimary)
  root.style.setProperty('--text-secondary', p.textSecondary)
  root.style.setProperty('--accent', p.accent)
  root.style.setProperty('--accent-primary', p.accent)
  root.style.setProperty('--border-primary', p.border)
}

function clearCustomPaletteFromCss() {
  if (typeof document === 'undefined') return
  const root = document.documentElement
  root.style.removeProperty('--bg-primary')
  root.style.removeProperty('--bg-secondary')
  root.style.removeProperty('--bg-tertiary')
  root.style.removeProperty('--bg-input')
  root.style.removeProperty('--text-primary')
  root.style.removeProperty('--text-secondary')
  root.style.removeProperty('--accent')
  root.style.removeProperty('--accent-primary')
  root.style.removeProperty('--border-primary')
}

export function setTheme(theme: string) {
  // Normalize legacy 'dark' / 'light' values
  let normalizedTheme = theme
  if (theme === 'dark') normalizedTheme = 'jetbrains-dark'
  if (theme === 'light') normalizedTheme = 'github-light'

  currentTheme = normalizedTheme

  if (typeof document !== 'undefined') {
    document.documentElement.setAttribute('data-theme', normalizedTheme)
    if (normalizedTheme === 'custom') {
      applyCustomPaletteToCss(customPalette)
    } else {
      clearCustomPaletteFromCss()
    }
  }

  if (typeof localStorage !== 'undefined') {
    localStorage.setItem('slopssh-theme', normalizedTheme)
  }
}

export function toggleTheme() {
  const isLight = currentTheme === 'github-light'
  setTheme(isLight ? 'jetbrains-dark' : 'github-light')
}

export function initTheme() {
  if (typeof localStorage !== 'undefined') {
    const savedPalette = localStorage.getItem('slopssh-custom-palette')
    if (savedPalette) {
      try {
        customPalette = { ...DEFAULT_CUSTOM_PALETTE, ...JSON.parse(savedPalette) }
      } catch (_) {}
    }

    const saved = localStorage.getItem('slopssh-theme')
    setTheme(saved || 'jetbrains-dark')
  }
}

export function persistTheme(theme: string) {
  setTheme(theme)
}

export function getActiveXtermTheme(): ITheme {
  return getTerminalTheme(currentTheme, customPalette)
}

let currentSettings = $state<{
  font_family: string
  font_size: number
  terminal_scrollback: number
  terminal_copy_on_select: boolean
}>({
  font_family: 'JetBrains Mono, monospace',
  font_size: 14,
  terminal_scrollback: 10000,
  terminal_copy_on_select: true,
})

export function getTerminalSettings() {
  return currentSettings
}

export function setTerminalSettings(settings: typeof currentSettings) {
  currentSettings = settings
}
