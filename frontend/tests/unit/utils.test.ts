import { describe, it, expect } from 'vitest'

function formatFileSize(bytes: number): string {
  if (bytes === 0) return '0 B'
  const units = ['B', 'KB', 'MB', 'GB', 'TB']
  const i = Math.floor(Math.log(bytes) / Math.log(1024))
  const size = bytes / Math.pow(1024, i)
  return `${size.toFixed(i === 0 ? 0 : 1)} ${units[i]}`
}

function formatDate(date: Date): string {
  return date.toLocaleDateString(undefined, {
    year: 'numeric',
    month: 'short',
    day: 'numeric',
    hour: '2-digit',
    minute: '2-digit',
  })
}

function getFileExtension(filename: string): string {
  const idx = filename.lastIndexOf('.')
  if (idx === -1 || idx === filename.length - 1) return ''
  return filename.slice(idx + 1).toLowerCase()
}

function isHiddenFile(filename: string): boolean {
  return filename.startsWith('.')
}

describe('formatFileSize', () => {
  it('formats 0 bytes', () => {
    expect(formatFileSize(0)).toBe('0 B')
  })

  it('formats bytes', () => {
    expect(formatFileSize(512)).toBe('512 B')
  })

  it('formats kilobytes', () => {
    expect(formatFileSize(1024)).toBe('1.0 KB')
  })

  it('formats megabytes', () => {
    expect(formatFileSize(1048576)).toBe('1.0 MB')
  })

  it('formats gigabytes', () => {
    expect(formatFileSize(1073741824)).toBe('1.0 GB')
  })

  it('formats large values', () => {
    expect(formatFileSize(1500000)).toBe('1.4 MB')
  })
})

describe('getFileExtension', () => {
  it('returns extension for normal files', () => {
    expect(getFileExtension('file.txt')).toBe('txt')
  })

  it('returns empty for no extension', () => {
    expect(getFileExtension('file')).toBe('')
  })

  it('handles dotfiles', () => {
    expect(getFileExtension('.gitignore')).toBe('gitignore')
  })

  it('handles multiple dots', () => {
    expect(getFileExtension('archive.tar.gz')).toBe('gz')
  })

  it('handles trailing dot', () => {
    expect(getFileExtension('file.')).toBe('')
  })
})

describe('isHiddenFile', () => {
  it('detects hidden files', () => {
    expect(isHiddenFile('.bashrc')).toBe(true)
  })

  it('detects visible files', () => {
    expect(isHiddenFile('file.txt')).toBe(false)
  })

  it('detects double dot as visible', () => {
    expect(isHiddenFile('..')).toBe(true)
  })
})
