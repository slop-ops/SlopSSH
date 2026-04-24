export interface ShortcutBinding {
  key: string
  ctrl?: boolean
  alt?: boolean
  shift?: boolean
  meta?: boolean
  description: string
  action: string
}

const DEFAULT_SHORTCUTS: ShortcutBinding[] = [
  { key: 't', ctrl: true, description: 'New terminal tab', action: 'new-tab' },
  { key: 'w', ctrl: true, description: 'Close current tab', action: 'close-tab' },
  { key: 'Tab', ctrl: true, description: 'Next tab', action: 'next-tab' },
  { key: 'Tab', ctrl: true, shift: true, description: 'Previous tab', action: 'prev-tab' },
  { key: 's', ctrl: true, description: 'Toggle sidebar', action: 'toggle-sidebar' },
  { key: ',', ctrl: true, description: 'Open settings', action: 'open-settings' },
  { key: 'n', ctrl: true, shift: true, description: 'New session', action: 'new-session' },
  { key: 'f', ctrl: true, shift: true, description: 'Toggle file browser', action: 'toggle-files' },
  { key: 'l', ctrl: true, shift: true, description: 'Toggle tools panel', action: 'toggle-tools' },
  { key: 'Plus', ctrl: true, description: 'Increase font size', action: 'font-increase' },
  { key: 'Minus', ctrl: true, description: 'Decrease font size', action: 'font-decrease' },
  { key: '0', ctrl: true, description: 'Reset font size', action: 'font-reset' },
  { key: 'Escape', description: 'Close dialog / Cancel', action: 'escape' },
  { key: 'F5', description: 'Refresh file browser', action: 'refresh' },
  { key: 'F11', description: 'Toggle fullscreen', action: 'toggle-fullscreen' },
]

type ActionHandler = (action: string) => void

let handlers: ActionHandler[] = []
let shortcuts: ShortcutBinding[] = [...DEFAULT_SHORTCUTS]
let enabled = true

export function getShortcuts(): ShortcutBinding[] {
  return [...shortcuts]
}

export function getDefaultShortcuts(): ShortcutBinding[] {
  return [...DEFAULT_SHORTCUTS]
}

export function updateShortcuts(newShortcuts: ShortcutBinding[]) {
  shortcuts = [...DEFAULT_SHORTCUTS, ...newShortcuts]
}

export function resetShortcuts() {
  shortcuts = [...DEFAULT_SHORTCUTS]
}

export function registerHandler(handler: ActionHandler) {
  handlers.push(handler)
  return () => {
    handlers = handlers.filter(h => h !== handler)
  }
}

export function setEnabled(value: boolean) {
  enabled = value
}

function matchesBinding(binding: ShortcutBinding, e: KeyboardEvent): boolean {
  if (binding.key !== e.key && binding.key !== e.code) return false
  if (!!binding.ctrl !== (e.ctrlKey || e.metaKey)) return false
  if (!!binding.alt !== e.altKey) return false
  if (!!binding.shift !== e.shiftKey) return false
  if (!!binding.meta !== e.metaKey) return false
  return true
}

function handleKeyDown(e: KeyboardEvent) {
  if (!enabled) return

  const target = e.target as HTMLElement
  if (target.tagName === 'INPUT' || target.tagName === 'TEXTAREA' || target.isContentEditable) {
    if (e.key !== 'Escape') return
  }

  for (const binding of shortcuts) {
    if (matchesBinding(binding, e)) {
      e.preventDefault()
      e.stopPropagation()
      for (const handler of handlers) {
        handler(binding.action)
      }
      return
    }
  }
}

if (typeof window !== 'undefined') {
  window.addEventListener('keydown', handleKeyDown, true)
}

export function serializeShortcuts(): string {
  return JSON.stringify(shortcuts.filter(s => !DEFAULT_SHORTCUTS.find(d => d.action === s.action)))
}

export function deserializeShortcuts(json: string) {
  try {
    const custom: ShortcutBinding[] = JSON.parse(json)
    shortcuts = [...DEFAULT_SHORTCUTS, ...custom]
  } catch {
    shortcuts = [...DEFAULT_SHORTCUTS]
  }
}
