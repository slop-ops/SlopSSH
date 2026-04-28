import { describe, it, expect, beforeEach } from 'vitest'
import {
  getShortcuts,
  getDefaultShortcuts,
  updateShortcuts,
  resetShortcuts,
  serializeShortcuts,
  deserializeShortcuts,
  setEnabled,
} from '$lib/utils/shortcuts'
import type { ShortcutBinding } from '$lib/utils/shortcuts'

describe('shortcuts', () => {
  beforeEach(() => {
    resetShortcuts()
  })

  describe('getDefaultShortcuts', () => {
    it('returns non-empty array', () => {
      const defaults = getDefaultShortcuts()
      expect(defaults.length).toBeGreaterThan(0)
    })

    it('each shortcut has required fields', () => {
      const defaults = getDefaultShortcuts()
      for (const s of defaults) {
        expect(s.key).toBeDefined()
        expect(s.description).toBeDefined()
        expect(s.action).toBeDefined()
        expect(typeof s.key).toBe('string')
        expect(typeof s.action).toBe('string')
      }
    })

    it('has expected default actions', () => {
      const defaults = getDefaultShortcuts()
      const actions = defaults.map(s => s.action)
      expect(actions).toContain('new-tab')
      expect(actions).toContain('close-tab')
      expect(actions).toContain('toggle-sidebar')
      expect(actions).toContain('escape')
    })

    it('returns a copy', () => {
      const a = getDefaultShortcuts()
      const b = getDefaultShortcuts()
      expect(a).not.toBe(b)
      expect(a).toEqual(b)
    })
  })

  describe('getShortcuts', () => {
    it('returns defaults after reset', () => {
      const shortcuts = getShortcuts()
      const defaults = getDefaultShortcuts()
      expect(shortcuts).toEqual(defaults)
    })

    it('returns a copy', () => {
      const a = getShortcuts()
      const b = getShortcuts()
      expect(a).not.toBe(b)
    })
  })

  describe('updateShortcuts', () => {
    it('adds custom shortcuts on top of defaults', () => {
      const custom: ShortcutBinding[] = [
        { key: 'k', ctrl: true, description: 'Custom action', action: 'custom-action' },
      ]
      updateShortcuts(custom)
      const shortcuts = getShortcuts()
      const actions = shortcuts.map(s => s.action)
      expect(actions).toContain('custom-action')
      expect(actions).toContain('new-tab')
    })

    it('handles empty custom shortcuts', () => {
      updateShortcuts([])
      const shortcuts = getShortcuts()
      const defaults = getDefaultShortcuts()
      expect(shortcuts.length).toBe(defaults.length)
    })
  })

  describe('resetShortcuts', () => {
    it('resets to defaults after update', () => {
      updateShortcuts([
        { key: 'x', description: 'Extra', action: 'extra' },
      ])
      expect(getShortcuts().map(s => s.action)).toContain('extra')
      resetShortcuts()
      expect(getShortcuts().map(s => s.action)).not.toContain('extra')
    })
  })

  describe('serializeShortcuts / deserializeShortcuts', () => {
    it('serializes only custom shortcuts', () => {
      const custom: ShortcutBinding[] = [
        { key: 'p', ctrl: true, description: 'Custom', action: 'custom-1' },
      ]
      updateShortcuts(custom)
      const serialized = serializeShortcuts()
      const parsed = JSON.parse(serialized)
      expect(parsed.length).toBe(1)
      expect(parsed[0].action).toBe('custom-1')
    })

    it('serializes empty when no custom', () => {
      const serialized = serializeShortcuts()
      expect(JSON.parse(serialized)).toEqual([])
    })

    it('deserializes custom shortcuts', () => {
      updateShortcuts([
        { key: 'p', ctrl: true, description: 'Custom', action: 'custom-1' },
      ])
      const serialized = serializeShortcuts()
      resetShortcuts()
      deserializeShortcuts(serialized)
      expect(getShortcuts().map(s => s.action)).toContain('custom-1')
    })

    it('handles invalid JSON gracefully', () => {
      deserializeShortcuts('not-valid-json')
      expect(getShortcuts().length).toBe(getDefaultShortcuts().length)
    })
  })

  describe('setEnabled', () => {
    it('does not throw', () => {
      expect(() => setEnabled(false)).not.toThrow()
      expect(() => setEnabled(true)).not.toThrow()
    })
  })
})
