export const STANDARD_MONOSPACE_FALLBACKS: string[] = [
  'JetBrains Mono',
  'Fira Code',
  'Cascadia Code',
  'Cascadia Mono',
  'Ubuntu Mono',
  'Ubuntu Sans Mono',
  'DejaVu Sans Mono',
  'Liberation Mono',
  'Consolas',
  'Menlo',
  'Monaco',
  'Courier New',
  'monospace',
]

/**
 * Ensures font name is wrapped in single quotes if it contains spaces or special characters
 * and is not already quoted.
 */
export function formatFontName(name: string): string {
  const trimmed = name.trim()
  if (!trimmed) return ''
  if (
    trimmed === 'monospace' ||
    trimmed === 'serif' ||
    trimmed === 'sans-serif' ||
    (trimmed.startsWith("'") && trimmed.endsWith("'")) ||
    (trimmed.startsWith('"') && trimmed.endsWith('"'))
  ) {
    return trimmed
  }
  if (trimmed.includes(' ')) {
    return `'${trimmed}'`
  }
  return trimmed
}

/**
 * Builds a bulletproof terminal CSS font-family stack.
 * Guarantees that even if the primary font is missing or uninstalled,
 * xterm.js will strictly fall back to monospace fonts and never to a
 * proportional variable-width font (which causes wide character gaps).
 */
export function buildTerminalFontFamily(selectedFont?: string): string {
  const fontStack: string[] = []
  const seenNames = new Set<string>()

  function addFont(raw: string) {
    const trimmed = raw.trim()
    if (!trimmed) return
    const cleanedKey = trimmed.replace(/['"]/g, '').toLowerCase()
    if (seenNames.has(cleanedKey)) return
    seenNames.add(cleanedKey)
    fontStack.push(formatFontName(trimmed))
  }

  if (selectedFont && selectedFont.trim()) {
    // If selectedFont has comma-separated list, parse each
    for (const part of selectedFont.split(',')) {
      addFont(part)
    }
  }

  for (const fallback of STANDARD_MONOSPACE_FALLBACKS) {
    addFont(fallback)
  }

  // Ensure generic 'monospace' is strictly at the end
  if (!seenNames.has('monospace')) {
    fontStack.push('monospace')
  }

  return fontStack.join(', ')
}

/**
 * Tests if a given font family is installed and available to render in the webview.
 */
export function isFontAvailable(fontName: string): boolean {
  if (typeof document === 'undefined') return true
  try {
    const canvas = document.createElement('canvas')
    const ctx = canvas.getContext('2d')
    if (!ctx) return true

    const testText = 'abcdefghijklmnopqrstuvwxyz0123456789!@#$%^&*()_+'
    const baselineFont = '72px monospace'
    ctx.font = baselineFont
    const baselineWidth = ctx.measureText(testText).width

    const testFont = `72px ${formatFontName(fontName)}, monospace`
    ctx.font = testFont
    const testWidth = ctx.measureText(testText).width

    // Also check against sans-serif baseline
    const sansBaseline = '72px sans-serif'
    ctx.font = sansBaseline
    const sansBaselineWidth = ctx.measureText(testText).width

    const sansTest = `72px ${formatFontName(fontName)}, sans-serif`
    ctx.font = sansTest
    const sansTestWidth = ctx.measureText(testText).width

    // If both match the fallback exactly, the custom font wasn't applied
    return testWidth !== baselineWidth || sansTestWidth !== sansBaselineWidth
  } catch {
    return true
  }
}
