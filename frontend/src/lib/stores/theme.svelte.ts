let currentTheme = $state<string>('dark')

export function getTheme(): string {
  return currentTheme
}

export function setTheme(theme: string) {
  currentTheme = theme
  if (typeof document !== 'undefined') {
    document.documentElement.setAttribute('data-theme', theme)
  }
}

export function toggleTheme() {
  setTheme(currentTheme === 'dark' ? 'light' : 'dark')
}

export function initTheme() {
  const saved = localStorage.getItem('slopssh-theme')
  setTheme(saved || 'dark')
}

export function persistTheme(theme: string) {
  localStorage.setItem('slopssh-theme', theme)
}

let currentSettings = $state<{ font_family: string; font_size: number; terminal_scrollback: number; terminal_copy_on_select: boolean }>({
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
