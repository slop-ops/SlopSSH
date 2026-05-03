import { describe, it, expect } from 'vitest'
import { darkTheme, lightTheme } from '$lib/terminal/themes'

const REQUIRED_COLORS = [
  'foreground',
  'background',
  'cursor',
  'cursorAccent',
  'selectionBackground',
  'black',
  'red',
  'green',
  'yellow',
  'blue',
  'magenta',
  'cyan',
  'white',
  'brightBlack',
  'brightRed',
  'brightGreen',
  'brightYellow',
  'brightBlue',
  'brightMagenta',
  'brightCyan',
  'brightWhite',
] as const

describe('terminal themes', () => {
  describe('darkTheme', () => {
    it('has a name', () => {
      expect(darkTheme.name).toBe('SlopSSH Dark')
    })

    it('has all required color properties', () => {
      for (const color of REQUIRED_COLORS) {
        expect(darkTheme[color]).toBeDefined()
        expect(typeof darkTheme[color]).toBe('string')
      }
    })

    it('has valid hex colors', () => {
      const hexRegex = /^#[0-9a-fA-F]{6}$/
      for (const color of REQUIRED_COLORS) {
        if (color === 'selectionBackground') continue
        expect(hexRegex.test(darkTheme[color])).toBe(true)
      }
    })

    it('has dark background', () => {
      expect(darkTheme.background).toMatch(/^#[0-3]/)
    })

    it('has light foreground', () => {
      expect(darkTheme.foreground).toMatch(/^#[d-fD-F]/)
    })
  })

  describe('lightTheme', () => {
    it('has a name', () => {
      expect(lightTheme.name).toBe('SlopSSH Light')
    })

    it('has all required color properties', () => {
      for (const color of REQUIRED_COLORS) {
        expect(lightTheme[color]).toBeDefined()
        expect(typeof lightTheme[color]).toBe('string')
      }
    })

    it('has valid hex colors', () => {
      const hexRegex = /^#[0-9a-fA-F]{6}$/
      for (const color of REQUIRED_COLORS) {
        if (color === 'selectionBackground') continue
        expect(hexRegex.test(lightTheme[color])).toBe(true)
      }
    })

    it('has light background', () => {
      expect(lightTheme.background).toMatch(/^#[e-fE-F]/)
    })
  })

  describe('theme differences', () => {
    it('dark and light have different backgrounds', () => {
      expect(darkTheme.background).not.toBe(lightTheme.background)
    })

    it('dark and light have different foregrounds', () => {
      expect(darkTheme.foreground).not.toBe(lightTheme.foreground)
    })

    it('both have same selection background', () => {
      expect(darkTheme.selectionBackground).toBe(lightTheme.selectionBackground)
    })
  })
})
