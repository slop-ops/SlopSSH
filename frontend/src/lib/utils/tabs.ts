/**
 * Tab utility functions for SlopSSH
 */

export interface TitledTab {
  title: string
}

export interface IdentifiableTab {
  id: string
}

/**
 * Computes the next sequential terminal title based on existing tabs.
 * Searches for titles matching `prefix N` and returns `${prefix} ${max + 1}`.
 * If no numbered tabs exist, returns `${prefix} 1`.
 */
export function getNextTerminalTitle(tabs: TitledTab[], prefix = 'Terminal'): string {
  const escaped = prefix.replace(/[.*+?^${}()|[\]\\]/g, '\\$&')
  const regex = new RegExp(`^${escaped}\\s*(\\d+)$`, 'i')
  let max = 0
  for (const tab of tabs) {
    const m = tab.title.match(regex)
    if (m) {
      const num = parseInt(m[1], 10)
      if (num > max) max = num
    }
  }
  return `${prefix} ${max + 1}`
}

/**
 * Reorders an array so that `itemToMoveId` is placed immediately after `targetId`.
 * If either ID is not found, or if both IDs are identical, returns a copy of the original array.
 */
export function placeAdjacent<T extends IdentifiableTab>(items: T[], targetId: string, itemToMoveId: string): T[] {
  const targetIdx = items.findIndex((i) => i.id === targetId)
  const moveIdx = items.findIndex((i) => i.id === itemToMoveId)
  if (targetIdx === -1 || moveIdx === -1 || targetIdx === moveIdx) return [...items]

  const item = items[moveIdx]
  const filtered = items.filter((i) => i.id !== itemToMoveId)
  const newTargetIdx = filtered.findIndex((i) => i.id === targetId)
  filtered.splice(newTargetIdx + 1, 0, item)
  return filtered
}
