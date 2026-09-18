import { describe, it, expect } from 'vitest'
import { getNextTerminalTitle, placeAdjacent } from '$lib/utils/tabs'

describe('getNextTerminalTitle', () => {
  it('returns Terminal 1 when tabs list is empty', () => {
    expect(getNextTerminalTitle([])).toBe('Terminal 1')
  })

  it('increments sequentially from existing tabs', () => {
    expect(getNextTerminalTitle([{ title: 'Terminal 1' }])).toBe('Terminal 2')
    expect(getNextTerminalTitle([{ title: 'Terminal 1' }, { title: 'Terminal 2' }])).toBe('Terminal 3')
  })

  it('handles gaps and unordered tabs by finding maximum', () => {
    expect(getNextTerminalTitle([{ title: 'Terminal 3' }, { title: 'Terminal 1' }])).toBe('Terminal 4')
  })

  it('ignores unrelated tab titles', () => {
    expect(getNextTerminalTitle([{ title: 'Custom Tab' }, { title: 'Local' }])).toBe('Terminal 1')
  })

  it('supports custom prefixes like Local Terminal', () => {
    expect(getNextTerminalTitle([], 'Local Terminal')).toBe('Local Terminal 1')
    expect(getNextTerminalTitle([{ title: 'Local Terminal 1' }], 'Local Terminal')).toBe('Local Terminal 2')
  })
})

describe('placeAdjacent', () => {
  it('places item immediately after target', () => {
    const tabs = [
      { id: '1', title: 'Terminal 1' },
      { id: '3', title: 'Terminal 3' },
      { id: '2', title: 'Terminal 2' },
    ]
    const result = placeAdjacent(tabs, '1', '2')
    expect(result.map((t) => t.id)).toEqual(['1', '2', '3'])
  })

  it('handles moving to end or beginning', () => {
    const tabs = [
      { id: '1', title: 'T1' },
      { id: '2', title: 'T2' },
      { id: '3', title: 'T3' },
    ]
    // Move T1 after T3
    expect(placeAdjacent(tabs, '3', '1').map((t) => t.id)).toEqual(['2', '3', '1'])
  })

  it('returns same array copy if ids are invalid or identical', () => {
    const tabs = [
      { id: '1', title: 'T1' },
      { id: '2', title: 'T2' },
    ]
    expect(placeAdjacent(tabs, '1', '1')).toEqual(tabs)
    expect(placeAdjacent(tabs, '1', 'nonexistent')).toEqual(tabs)
    expect(placeAdjacent(tabs, 'nonexistent', '1')).toEqual(tabs)
  })
})
