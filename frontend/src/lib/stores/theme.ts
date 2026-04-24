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
  const saved = localStorage.getItem('muon-theme')
  setTheme(saved || 'dark')
}

export function persistTheme(theme: string) {
  localStorage.setItem('muon-theme', theme)
}
