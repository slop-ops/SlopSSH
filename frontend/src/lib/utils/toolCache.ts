const cache = new Map<string, { data: unknown; timestamp: number }>()

const DEFAULT_TTL = 60_000

export function getCached<T>(key: string, ttl: number = DEFAULT_TTL): T | null {
  const entry = cache.get(key)
  if (!entry) return null
  if (Date.now() - entry.timestamp > ttl) {
    cache.delete(key)
    return null
  }
  return entry.data as T
}

export function setCache(key: string, data: unknown): void {
  cache.set(key, { data, timestamp: Date.now() })
}

export function clearCache(key: string): void {
  cache.delete(key)
}

export function clearSessionCache(sessionId: string): void {
  const prefix = `${sessionId}:`
  for (const key of cache.keys()) {
    if (key.startsWith(prefix)) {
      cache.delete(key)
    }
  }
}
