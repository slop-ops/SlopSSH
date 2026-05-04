import en from '../../i18n/en.json'

type TranslationMap = Record<string, Record<string, string>>

const RTL_LOCALES = new Set(['ar', 'he', 'fa', 'ur'])

const translations: Record<string, TranslationMap> = { en: en as unknown as TranslationMap }

let currentLocale = 'en'

const loadedLocales = new Set<string>(['en'])

export function setLocale(locale: string) {
  currentLocale = locale
}

export function getLocale(): string {
  return currentLocale
}

export function isRTL(locale?: string): boolean {
  return RTL_LOCALES.has(locale ?? currentLocale)
}

export function getTextDirection(locale?: string): 'ltr' | 'rtl' {
  return isRTL(locale) ? 'rtl' : 'ltr'
}

export async function loadLocale(locale: string) {
  if (loadedLocales.has(locale)) {
    currentLocale = locale
    applyDirection()
    return
  }

  try {
    const mod = await import(`../../i18n/${locale}.json`)
    translations[locale] = mod.default as TranslationMap
    loadedLocales.add(locale)
    currentLocale = locale
    applyDirection()
  } catch {
    console.warn(`Locale ${locale} not found, falling back to English`)
    currentLocale = 'en'
    applyDirection()
  }
}

function applyDirection() {
  if (typeof document === 'undefined') return
  const dir = getTextDirection()
  document.documentElement.setAttribute('dir', dir)
}

export function t(path: string, params?: Record<string, string>): string {
  const parts = path.split('.')
  let value: unknown = translations[currentLocale] ?? translations['en']

  for (const part of parts) {
    if (value && typeof value === 'object' && part in value) {
      value = (value as Record<string, unknown>)[part]
    } else {
      value = translations['en']
      for (const p of parts) {
        if (value && typeof value === 'object' && p in value) {
          value = (value as Record<string, unknown>)[p]
        } else {
          return path
        }
      }
      break
    }
  }

  if (typeof value !== 'string') return path

  if (params) {
    return value.replace(/\{(\w+)\}/g, (_, key) => params[key] ?? `{${key}}`)
  }

  return value
}

export function useTranslations() {
  function tFn(path: string, params?: Record<string, string>): string {
    return t(path, params)
  }
  return { t: tFn, locale: currentLocale }
}
