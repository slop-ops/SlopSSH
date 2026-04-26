<script lang="ts">
  import * as api from '$lib/api/invoke'
  import { t } from '$lib/utils/i18n'

  let { sessionId }: { sessionId: string } = $props()

  let cpuHistory = $state<number[]>([])
  let memHistory = $state<number[]>([])
  let swapHistory = $state<number[]>([])
  let cpu = $state(0)
  let mem = $state(0)
  let swap = $state(0)
  let memTotal = $state('')
  let memUsed = $state('')
  let loading = $state(false)
  let error = $state('')
  let running = $state(false)
  let interval: ReturnType<typeof setInterval> | undefined

  $effect(() => {
    return () => {
      if (interval) clearInterval(interval)
    }
  })

  async function sample() {
    try {
      const result = await api.remoteExec(
        sessionId,
        `grep 'cpu ' /proc/stat 2>/dev/null && free 2>/dev/null | grep -E 'Mem|Swap'`,
        5,
      )
      parseStats(result.stdout)
    } catch (e) {
      error = String(e)
      stop()
    }
  }

  function parseStats(output: string) {
    const lines = output.split('\n').filter((l: string) => l.trim())
    for (const line of lines) {
      if (line.startsWith('cpu ')) {
        const parts = line.trim().split(/\s+/)
        const idle = parseInt(parts[4]) || 0
        const total = parts.slice(1).reduce((s: number, v: string) => s + (parseInt(v) || 0), 0)
        const used = total - idle
        cpu = total > 0 ? Math.round((used / total) * 100) : 0
      }
      if (line.startsWith('Mem:')) {
        const parts = line.trim().split(/\s+/)
        const total = parseInt(parts[1]) || 1
        const used = parseInt(parts[2]) || 0
        mem = Math.round((used / total) * 100)
        memTotal = formatBytes(total * 1024)
        memUsed = formatBytes(used * 1024)
      }
      if (line.startsWith('Swap:')) {
        const parts = line.trim().split(/\s+/)
        const total = parseInt(parts[1]) || 1
        const used = parseInt(parts[2]) || 0
        swap = total > 0 ? Math.round((used / total) * 100) : 0
      }
    }

    cpuHistory = [...cpuHistory, cpu].slice(-60)
    memHistory = [...memHistory, mem].slice(-60)
    swapHistory = [...swapHistory, swap].slice(-60)
  }

  function formatBytes(bytes: number): string {
    if (bytes === 0) return '0 B'
    const units = ['B', 'KB', 'MB', 'GB', 'TB']
    const i = Math.floor(Math.log(bytes) / Math.log(1024))
    return (bytes / Math.pow(1024, i)).toFixed(1) + ' ' + units[i]
  }

  function start() {
    running = true
    sample()
    interval = setInterval(sample, 2000)
  }

  function stop() {
    running = false
    if (interval) clearInterval(interval)
    interval = undefined
  }

  function sparklinePath(data: number[], height: number = 40): string {
    if (data.length < 2) return ''
    const width = 100
    const step = width / (data.length - 1)
    return data
      .map((v, i) => {
        const x = i * step
        const y = height - (v / 100) * height
        return `${i === 0 ? 'M' : 'L'}${x},${y}`
      })
      .join(' ')
  }
</script>

<div class="system-load" role="region" aria-label="System load monitoring">
  <div class="toolbar">
    {#if running}
      <button class="action-btn active" onclick={stop}>{t('tools.stop')}</button>
    {:else}
      <button class="action-btn primary" onclick={start}>{t('tools.startMonitoring')}</button>
    {/if}
    <button class="action-btn" onclick={() => { cpuHistory = []; memHistory = []; swapHistory = [] }}>{t('tools.reset')}</button>
  </div>

  {#if error}
    <div class="error">{error}</div>
  {/if}

  <div class="gauges">
    <div class="gauge">
      <div class="gauge-header">
        <span class="gauge-label">{t('tools.cpu')}</span>
        <span class="gauge-value">{cpu}%</span>
      </div>
      <div class="gauge-bar-container">
        <div class="gauge-bar cpu" style:width="{cpu}%"></div>
      </div>
      <svg class="sparkline" viewBox="0 0 100 40" preserveAspectRatio="none">
        <path d={sparklinePath(cpuHistory)} fill="none" stroke="#61afef" stroke-width="1.5" vector-effect="non-scaling-stroke" />
      </svg>
    </div>

    <div class="gauge">
      <div class="gauge-header">
        <span class="gauge-label">{t('tools.memory')}</span>
        <span class="gauge-value">{mem}% ({memUsed} / {memTotal})</span>
      </div>
      <div class="gauge-bar-container">
        <div class="gauge-bar mem" style:width="{mem}%"></div>
      </div>
      <svg class="sparkline" viewBox="0 0 100 40" preserveAspectRatio="none">
        <path d={sparklinePath(memHistory)} fill="none" stroke="#98c379" stroke-width="1.5" vector-effect="non-scaling-stroke" />
      </svg>
    </div>

    <div class="gauge">
      <div class="gauge-header">
        <span class="gauge-label">{t('tools.swap')}</span>
        <span class="gauge-value">{swap}%</span>
      </div>
      <div class="gauge-bar-container">
        <div class="gauge-bar swap" style:width="{swap}%"></div>
      </div>
      <svg class="sparkline" viewBox="0 0 100 40" preserveAspectRatio="none">
        <path d={sparklinePath(swapHistory)} fill="none" stroke="#e5c07b" stroke-width="1.5" vector-effect="non-scaling-stroke" />
      </svg>
    </div>
  </div>

  {#if !running && cpuHistory.length === 0}
    <div class="empty">{t('tools.clickStart')}</div>
  {/if}
</div>

<style>
  .system-load {
    display: flex;
    flex-direction: column;
    height: 100%;
    background: var(--bg-tertiary);
  }

  .toolbar {
    display: flex;
    gap: 8px;
    padding: 8px 12px;
    border-bottom: 1px solid var(--border-primary);
  }

  .action-btn {
    background: transparent;
    border: 1px solid var(--border-primary);
    color: var(--text-secondary);
    padding: 5px 12px;
    border-radius: 4px;
    cursor: pointer;
    font-size: 12px;
  }

  .action-btn:hover {
    background: var(--bg-hover);
    color: var(--text-primary);
  }

  .action-btn.primary {
    background: var(--accent-bg);
    border-color: var(--border-active);
    color: var(--accent);
  }

  .action-btn.primary:hover {
    background: var(--accent-bg);
  }

  .action-btn.active {
    background: var(--error-bg);
    border-color: var(--error);
    color: var(--error);
  }

  .error {
    background: var(--error-bg);
    color: var(--error);
    padding: 6px 12px;
    font-size: 12px;
  }

  .gauges {
    padding: 12px;
    display: flex;
    flex-direction: column;
    gap: 16px;
  }

  .gauge {
    display: flex;
    flex-direction: column;
    gap: 4px;
  }

  .gauge-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
  }

  .gauge-label {
    font-size: 11px;
    color: var(--text-secondary);
    text-transform: uppercase;
    letter-spacing: 0.5px;
    font-weight: 600;
  }

  .gauge-value {
    font-size: 12px;
    color: var(--text-primary);
    font-family: 'JetBrains Mono', monospace;
  }

  .gauge-bar-container {
    height: 6px;
    background: var(--bg-hover);
    border-radius: 3px;
    overflow: hidden;
  }

  .gauge-bar {
    height: 100%;
    border-radius: 3px;
    transition: width 0.5s ease;
  }

  .gauge-bar.cpu {
    background: linear-gradient(90deg, var(--info), var(--accent));
  }

  .gauge-bar.mem {
    background: linear-gradient(90deg, var(--success), var(--success));
  }

  .gauge-bar.swap {
    background: linear-gradient(90deg, var(--warning), var(--warning));
  }

  .sparkline {
    width: 100%;
    height: 40px;
    background: var(--bg-secondary);
    border-radius: 4px;
    margin-top: 2px;
  }

  .empty {
    text-align: center;
    color: var(--text-tertiary);
    padding: 24px;
    font-size: 13px;
    flex: 1;
    display: flex;
    align-items: center;
    justify-content: center;
  }
</style>
