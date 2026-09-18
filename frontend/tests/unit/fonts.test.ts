import { describe, it, expect } from 'vitest'
import {
  formatFontName,
  buildTerminalFontFamily,
  STANDARD_MONOSPACE_FALLBACKS,
} from '$lib/utils/fonts'

describe('fonts utility', () => {
  it('formats font names with spaces using quotes', () => {
    expect(formatFontName('Ubuntu Mono')).toBe("'Ubuntu Mono'")
    expect(formatFontName('JetBrains Mono')).toBe("'JetBrains Mono'")
    expect(formatFontName('Fira Code')).toBe("'Fira Code'")
  })

  it('keeps single-word fonts without quotes', () => {
    expect(formatFontName('Consolas')).toBe('Consolas')
    expect(formatFontName('Menlo')).toBe('Menlo')
    expect(formatFontName('monospace')).toBe('monospace')
  })

  it('preserves already quoted font names', () => {
    expect(formatFontName("'Ubuntu Mono'")).toBe("'Ubuntu Mono'")
    expect(formatFontName('"Ubuntu Mono"')).toBe('"Ubuntu Mono"')
  })

  it('builds font family with custom font placed first', () => {
    const stack = buildTerminalFontFamily('Ubuntu Mono')
    expect(stack.startsWith("'Ubuntu Mono'")).toBe(true)
    expect(stack).toContain('monospace')
    expect(stack).toContain('Consolas')
    expect(stack).toContain('Menlo')
  })

  it('builds font family without duplicates if custom font is in standard list', () => {
    const stack = buildTerminalFontFamily('DejaVu Sans Mono')
    const matches = stack.match(/DejaVu Sans Mono/g)
    expect(matches?.length).toBe(1)
  })

  it('builds clean fallback stack when font is undefined or empty', () => {
    const stack = buildTerminalFontFamily('')
    expect(stack.length).toBeGreaterThan(0)
    expect(stack.endsWith('monospace')).toBe(true)
    expect(stack).toContain("'JetBrains Mono'")
  })

  it('correctly handles comma-separated user inputs', () => {
    const stack = buildTerminalFontFamily('My Font, Consolas')
    expect(stack.startsWith("'My Font', Consolas")).toBe(true)
  })
})
