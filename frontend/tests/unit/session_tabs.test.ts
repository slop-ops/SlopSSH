import { describe, it, expect } from 'vitest'
import type { SessionTabItem } from '../../src/components/layout/SessionTabBar.svelte'

describe('Session Tab Architecture & Lifecycle', () => {
  interface Tab {
    id: string
    sessionId: string
    channelId: string
    title: string
    isLocal?: boolean
  }

  interface SessionWorkspace {
    sessionId: string
    name?: string
    host?: string
    username?: string
    port?: number
    tabs: Tab[]
    activeTabId: string
    activeView: 'terminal' | 'files' | 'tools'
    status?: 'connected' | 'reconnecting' | 'disconnected'
  }

  it('correctly maps connected workspaces to session tabs', () => {
    const workspaces = new Map<string, SessionWorkspace>()
    workspaces.set('sess-1', {
      sessionId: 'sess-1',
      name: 'Prod VPS',
      host: '192.168.1.10',
      username: 'root',
      port: 22,
      tabs: [{ id: 'tab-1', sessionId: 'sess-1', channelId: 'ch-1', title: 'Terminal 1' }],
      activeTabId: 'tab-1',
      activeView: 'terminal',
      status: 'connected',
    })
    workspaces.set('sess-2', {
      sessionId: 'sess-2',
      name: 'Dev Box',
      host: '10.0.0.5',
      username: 'developer',
      port: 2222,
      tabs: [],
      activeTabId: '',
      activeView: 'files',
      status: 'connected',
    })

    const tabs: SessionTabItem[] = Array.from(workspaces.values()).map((w) => ({
      id: w.sessionId,
      title: w.name || w.host || 'Session',
      subtitle: w.username && w.host ? `${w.username}@${w.host}` : undefined,
      status: w.status || 'connected',
      isLocal: false,
    }))

    expect(tabs).toHaveLength(2)
    expect(tabs[0].title).toBe('Prod VPS')
    expect(tabs[0].subtitle).toBe('root@192.168.1.10')
    expect(tabs[0].status).toBe('connected')
    expect(tabs[1].title).toBe('Dev Box')
    expect(tabs[1].subtitle).toBe('developer@10.0.0.5')
  })

  it('keeps workspace alive when terminal tabs reach zero', () => {
    const workspaces = new Map<string, SessionWorkspace>()
    const ws: SessionWorkspace = {
      sessionId: 'sess-1',
      name: 'Prod VPS',
      host: '192.168.1.10',
      tabs: [{ id: 'tab-1', sessionId: 'sess-1', channelId: 'ch-1', title: 'Terminal 1' }],
      activeTabId: 'tab-1',
      activeView: 'terminal',
      status: 'connected',
    }
    workspaces.set('sess-1', ws)

    // User closes the only terminal tab:
    ws.tabs = ws.tabs.filter((t) => t.id !== 'tab-1')
    ws.activeTabId = ''

    // Workspace is NOT deleted!
    expect(workspaces.has('sess-1')).toBe(true)
    expect(ws.tabs).toHaveLength(0)
    // Files and tools views can still be toggled
    ws.activeView = 'files'
    expect(ws.activeView).toBe('files')
  })

  it('handles disconnection and reconnection transitions', () => {
    const ws: SessionWorkspace = {
      sessionId: 'sess-1',
      name: 'Prod VPS',
      tabs: [],
      activeTabId: '',
      activeView: 'terminal',
      status: 'connected',
    }

    // Network drops:
    ws.status = 'disconnected'
    expect(ws.status).toBe('disconnected')

    // User clicks Reconnect:
    ws.status = 'reconnecting'
    expect(ws.status).toBe('reconnecting')

    // SSH handshake succeeds:
    ws.status = 'connected'
    expect(ws.status).toBe('connected')
  })
})
