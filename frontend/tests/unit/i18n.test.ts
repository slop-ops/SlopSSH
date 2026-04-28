import { describe, it, expect, beforeEach } from 'vitest'
import { setLocale, getLocale, isRTL, getTextDirection, t } from '$lib/utils/i18n'

describe('i18n', () => {
  beforeEach(() => {
    setLocale('en')
  })

  describe('setLocale / getLocale', () => {
    it('defaults to en', () => {
      expect(getLocale()).toBe('en')
    })

    it('sets and gets locale', () => {
      setLocale('de')
      expect(getLocale()).toBe('de')
    })

    it('can set back to en', () => {
      setLocale('de')
      setLocale('en')
      expect(getLocale()).toBe('en')
    })
  })

  describe('isRTL', () => {
    it('returns false for English', () => {
      expect(isRTL('en')).toBe(false)
    })

    it('returns true for Arabic', () => {
      expect(isRTL('ar')).toBe(true)
    })

    it('returns true for Hebrew', () => {
      expect(isRTL('he')).toBe(true)
    })

    it('returns true for Farsi', () => {
      expect(isRTL('fa')).toBe(true)
    })

    it('returns true for Urdu', () => {
      expect(isRTL('ur')).toBe(true)
    })

    it('returns false for German', () => {
      expect(isRTL('de')).toBe(false)
    })

    it('uses current locale when no argument', () => {
      setLocale('ar')
      expect(isRTL()).toBe(true)
      setLocale('en')
      expect(isRTL()).toBe(false)
    })
  })

  describe('getTextDirection', () => {
    it('returns ltr for English', () => {
      expect(getTextDirection('en')).toBe('ltr')
    })

    it('returns rtl for Arabic', () => {
      expect(getTextDirection('ar')).toBe('rtl')
    })

    it('returns ltr for French', () => {
      expect(getTextDirection('fr')).toBe('ltr')
    })
  })

  describe('t', () => {
    it('returns path when no translations loaded', () => {
      expect(t('nonexistent.key')).toBe('nonexistent.key')
    })

    it('returns path for empty string', () => {
      expect(t('')).toBe('')
    })

    it('handles nested paths', () => {
      expect(t('a.b.c')).toBe('a.b.c')
    })
  })
})
