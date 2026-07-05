<script lang="ts">
  import { onDestroy } from 'svelte'
  import * as api from '$lib/api/invoke'
  import { t } from '$lib/utils/i18n'

  let { sessionId }: { sessionId: string } = $props()

  let cpuHistory = $state<number[]>([])
  let memHistory = $state<number[]>([])
  let swapHistory = $state<number[]>([])
  let netRxHistory = $state<number[]>([])
  let netTxHistory = $state<number[]>([])
  let diskReadHistory = $state<number[]>([])
  let diskWriteHistory = $state<number[]>([])

  let cpu = $state(0)
  let mem = $state(0)
  let swap = $state(0)
  let memTotal = $state('')
  let memUsed = $state('')
  let netRx = $state(0)
  let netTx = $state(0)
  let diskRead = $state(0)
  let diskWrite = $state(0)

  let loading = $state(false)
  let error = $state('')
  let running = $state(false)
  let interval: ReturnType<typeof setInterval> | undefined = $state()

  let prevCpuIdle = $state(0)
  let prevCpuTotal = $state(0)
  let prevNetRx = $state(0)
  let prevNetTx = $state(0)
  let prevDiskRead = $state(0)
  let prevDiskWrite = $state(0)
  let hasPrevSample = $state(false)

  let tooltipX = $state(0)
  let tooltipY = $state(0)
  let tooltipValue = $state('')
  let tooltipVisible = $state(false)
  let tooltipMetric = $state('')

  let cpuCanvas: HTMLCanvasElement | undefined = $state()
  let memCanvas: HTMLCanvasElement | undefined = $state()
  let netCanvas: HTMLCanvasElement | undefined = $state()
  let diskCanvas: HTMLCanvasElement | undefined = $state()

  function clearPolling() {
    if (interval) {
      clearInterval(interval)
      interval = undefined
    }
  }

  async function sample() {
    if (!sessionId) return
    try {
      const result = await api.remoteExec(
        sessionId,
        `grep 'cpu ' /proc/stat 2>/dev/null; free 2>/dev/null | grep -E 'Mem|Swap'; cat /proc/net/dev 2>/dev/null | grep -E 'eth|ens|enp|wlan|wlp'; cat /proc/diskstats 2>/dev/null | grep -E 'sd[a-z]|nvme[0-9]n[0-9] ' | head -5`,
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

    let newCpuIdle = 0
    let newCpuTotal = 0
    let newNetRx = 0
    let newNetTx = 0
    let newDiskRead = 0
    let newDiskWrite = 0

    for (const line of lines) {
      if (line.startsWith('cpu ')) {
        const parts = line.trim().split(/\s+/)
        const idle = parseInt(parts[4]) || 0
        const total = parts.slice(1).reduce((s: number, v: string) => s + (parseInt(v) || 0), 0)
        newCpuIdle = idle
        newCpuTotal = total
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
      if (line.match(/^\s*(eth|ens|enp|wlan|wlp)/)) {
        const parts = line.trim().split(/\s+/)
        if (parts.length >= 10) {
          newNetRx += parseInt(parts[1]) || 0
          newNetTx += parseInt(parts[9]) || 0
        }
      }
      if (line.match(/^\s*\d+\s+\d+\s+(sd[a-z]|nvme)/)) {
        const parts = line.trim().split(/\s+/)
        if (parts.length >= 14) {
          newDiskRead += parseInt(parts[5]) || 0
          newDiskWrite += parseInt(parts[9]) || 0
        }
      }
    }

    if (hasPrevSample) {
      const cpuDelta = newCpuTotal - prevCpuTotal
      const cpuIdleDelta = newCpuIdle - prevCpuIdle
      cpu = cpuDelta > 0 ? Math.round(((cpuDelta - cpuIdleDelta) / cpuDelta) * 100) : 0

      const netRxDelta = newNetRx - prevNetRx
      const netTxDelta = newNetTx - prevNetTx
      netRx = Math.max(0, netRxDelta / 2)
      netTx = Math.max(0, netTxDelta / 2)

      const diskReadDelta = newDiskRead - prevDiskRead
      const diskWriteDelta = newDiskWrite - prevDiskWrite
      diskRead = Math.max(0, diskReadDelta / 2)
      diskWrite = Math.max(0, diskWriteDelta / 2)
    }

    prevCpuIdle = newCpuIdle
    prevCpuTotal = newCpuTotal
    prevNetRx = newNetRx
    prevNetTx = newNetTx
    prevDiskRead = newDiskRead
    prevDiskWrite = newDiskWrite
    hasPrevSample = true

    cpuHistory = [...cpuHistory, cpu].slice(-60)
    memHistory = [...memHistory, mem].slice(-60)
    swapHistory = [...swapHistory, swap].slice(-60)
    netRxHistory = [...netRxHistory, netRx].slice(-60)
    netTxHistory = [...netTxHistory, netTx].slice(-60)
    diskReadHistory = [...diskReadHistory, diskRead].slice(-60)
    diskWriteHistory = [...diskWriteHistory, diskWrite].slice(-60)

    drawAllGraphs()
  }

  function formatBytes(bytes: number): string {
    if (bytes === 0) return '0 B'
    const units = ['B', 'KB', 'MB', 'GB', 'TB']
    const i = Math.floor(Math.log(bytes) / Math.log(1024))
    return (bytes / Math.pow(1024, i)).toFixed(1) + ' ' + units[i]
  }

  function formatRate(bytesPerSec: number): string {
    return formatBytes(bytesPerSec) + '/s'
  }

  function getStats(data: number[]): { min: number; max: number; avg: number } {
    if (data.length === 0) return { min: 0, max: 0, avg: 0 }
    const min = Math.min(...data)
    const max = Math.max(...data)
    const avg = Math.round(data.reduce((s, v) => s + v, 0) / data.length)
    return { min, max, avg }
  }

  function drawGraph(
    canvas: HTMLCanvasElement,
    data: number[],
    maxVal: number,
    color: string,
    fillColor: string,
  ) {
    const ctx = canvas.getContext('2d')
    if (!ctx || data.length < 2) return

    const dpr = window.devicePixelRatio || 1
    const rect = canvas.getBoundingClientRect()
    canvas.width = rect.width * dpr
    canvas.height = rect.height * dpr
    ctx.scale(dpr, dpr)

    const w = rect.width
    const h = rect.height
    const padLeft = 40
    const padRight = 10
    const padTop = 10
    const padBottom = 20
    const graphW = w - padLeft - padRight
    const graphH = h - padTop - padBottom

    ctx.clearRect(0, 0, w, h)

    ctx.strokeStyle = 'rgba(128,128,128,0.15)'
    ctx.lineWidth = 1
    for (let i = 0; i <= 4; i++) {
      const y = padTop + (graphH / 4) * i
      ctx.beginPath()
      ctx.moveTo(padLeft, y)
      ctx.lineTo(padLeft + graphW, y)
      ctx.stroke()

      ctx.fillStyle = 'rgba(128,128,128,0.5)'
      ctx.font = '10px monospace'
      ctx.textAlign = 'right'
      const val = Math.round(maxVal - (maxVal / 4) * i)
      ctx.fillText(maxVal > 1000 ? formatRate(val) : val + '%', padLeft - 4, y + 3)
    }

    const step = graphW / (data.length - 1)
    const points: [number, number][] = data.map((v, i) => [
      padLeft + i * step,
      padTop + graphH - (v / maxVal) * graphH,
    ])

    ctx.beginPath()
    ctx.moveTo(points[0][0], points[0][1])
    for (let i = 1; i < points.length; i++) {
      ctx.lineTo(points[i][0], points[i][1])
    }
    ctx.strokeStyle = color
    ctx.lineWidth = 1.5
    ctx.stroke()

    ctx.lineTo(points[points.length - 1][0], padTop + graphH)
    ctx.lineTo(points[0][0], padTop + graphH)
    ctx.closePath()
    ctx.fillStyle = fillColor
    ctx.fill()

    ctx.fillStyle = 'rgba(128,128,128,0.4)'
    ctx.font = '9px monospace'
    ctx.textAlign = 'center'
    const timeLabels = [0, Math.floor(data.length / 2), data.length - 1]
    for (const idx of timeLabels) {
      const x = padLeft + idx * step
      const secsAgo = (data.length - 1 - idx) * 2
      ctx.fillText(secsAgo === 0 ? 'now' : `-${secsAgo}s`, x, h - 4)
    }
  }

  function drawAllGraphs() {
    if (cpuCanvas) drawGraph(cpuCanvas, cpuHistory, 100, '#61afef', 'rgba(97,175,239,0.1)')
    if (memCanvas) drawGraph(memCanvas, memHistory, 100, '#98c379', 'rgba(152,195,121,0.1)')
    if (netCanvas) {
      const maxNet = Math.max(...netRxHistory, ...netTxHistory, 1024)
      drawGraph(netCanvas, netRxHistory, maxNet, '#61afef', 'rgba(97,175,239,0.1)')
      const ctx = netCanvas.getContext('2d')
      if (ctx) {
        const dpr = window.devicePixelRatio || 1
        const rect = netCanvas.getBoundingClientRect()
        const w = rect.width
        const h = rect.height
        const padLeft = 40
        const padRight = 10
        const padTop = 10
        const padBottom = 20
        const graphW = w - padLeft - padRight
        const graphH = h - padTop - padBottom
        const step = graphW / (netTxHistory.length - 1)
        const points: [number, number][] = netTxHistory.map((v, i) => [
          padLeft + i * step,
          padTop + graphH - (v / maxNet) * graphH,
        ])
        ctx.beginPath()
        ctx.moveTo(points[0][0], points[0][1])
        for (let i = 1; i < points.length; i++) {
          ctx.lineTo(points[i][0], points[i][1])
        }
        ctx.strokeStyle = '#e5c07b'
        ctx.lineWidth = 1.5
        ctx.stroke()
      }
    }
    if (diskCanvas) {
      const maxDisk = Math.max(...diskReadHistory, ...diskWriteHistory, 1024)
      drawGraph(diskCanvas, diskReadHistory, maxDisk, '#98c379', 'rgba(152,195,121,0.1)')
      const ctx = diskCanvas.getContext('2d')
      if (ctx) {
        const rect = diskCanvas.getBoundingClientRect()
        const w = rect.width
        const h = rect.height
        const padLeft = 40
        const padRight = 10
        const padTop = 10
        const padBottom = 20
        const graphW = w - padLeft - padRight
        const graphH = h - padTop - padBottom
        const step = graphW / (diskWriteHistory.length - 1)
        const points: [number, number][] = diskWriteHistory.map((v, i) => [
          padLeft + i * step,
          padTop + graphH - (v / maxDisk) * graphH,
        ])
        ctx.beginPath()
        ctx.moveTo(points[0][0], points[0][1])
        for (let i = 1; i < points.length; i++) {
          ctx.lineTo(points[i][0], points[i][1])
        }
        ctx.strokeStyle = '#e06c75'
        ctx.lineWidth = 1.5
        ctx.stroke()
      }
    }
  }

  function handleGraphHover(
    e: MouseEvent,
    data: number[],
    maxVal: number,
    metric: string,
    isRate: boolean,
  ) {
    const canvas = e.currentTarget as HTMLCanvasElement
    const rect = canvas.getBoundingClientRect()
    const padLeft = 40
    const padRight = 10
    const graphW = rect.width - padLeft - padRight
    const x = e.clientX - rect.left - padLeft
    const idx = Math.round((x / graphW) * (data.length - 1))

    if (idx >= 0 && idx < data.length) {
      tooltipX = e.clientX - rect.left
      tooltipY = e.clientY - rect.top - 30
      tooltipValue = isRate ? formatRate(data[idx]) : data[idx] + '%'
      tooltipVisible = true
      tooltipMetric = metric
    } else {
      tooltipVisible = false
    }
  }

  function hideTooltip() {
    tooltipVisible = false
  }

  function start() {
    running = true
    sample()
    clearPolling()
    interval = setInterval(sample, 2000)
  }

  function stop() {
    running = false
    clearPolling()
  }

  function resetHistory() {
    cpuHistory = []
    memHistory = []
    swapHistory = []
    netRxHistory = []
    netTxHistory = []
    diskReadHistory = []
    diskWriteHistory = []
    hasPrevSample = false
    drawAllGraphs()
  }

  $effect(() => {
    drawAllGraphs()
  })

  onDestroy(() => {
    clearPolling()
  })
</script>

<div class="system-load" role="region" aria-label="System load monitoring">
  <div class="toolbar">
    {#if running}
      <button class="action-btn active" onclick={stop}>{t('tools.stop')}</button>
    {:else}
      <button class="action-btn primary" onclick={start}>{t('tools.startMonitoring')}</button>
    {/if}
    <button class="action-btn" onclick={resetHistory}>{t('tools.reset')}</button>
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
      {#if cpuHistory.length > 1}
        <div class="graph-container">
          <canvas
            bind:this={cpuCanvas}
            class="graph-canvas"
            onmousemove={(e) => handleGraphHover(e, cpuHistory, 100, 'CPU', false)}
            onmouseleave={hideTooltip}
          ></canvas>
          {#if tooltipVisible && tooltipMetric === 'CPU'}
            <div class="tooltip" style:left="{tooltipX}px" style:top="{tooltipY}px">
              {tooltipValue}
            </div>
          {/if}
        </div>
      {/if}
      {#if cpuHistory.length > 0}
        {@const s = getStats(cpuHistory)}
        <div class="stats">
          <span>min: {s.min}%</span>
          <span>avg: {s.avg}%</span>
          <span>max: {s.max}%</span>
        </div>
      {/if}
    </div>

    <div class="gauge">
      <div class="gauge-header">
        <span class="gauge-label">{t('tools.memory')}</span>
        <span class="gauge-value">{mem}% ({memUsed} / {memTotal})</span>
      </div>
      <div class="gauge-bar-container">
        <div class="gauge-bar mem" style:width="{mem}%"></div>
      </div>
      {#if memHistory.length > 1}
        <div class="graph-container">
          <canvas
            bind:this={memCanvas}
            class="graph-canvas"
            onmousemove={(e) => handleGraphHover(e, memHistory, 100, 'Memory', false)}
            onmouseleave={hideTooltip}
          ></canvas>
          {#if tooltipVisible && tooltipMetric === 'Memory'}
            <div class="tooltip" style:left="{tooltipX}px" style:top="{tooltipY}px">
              {tooltipValue}
            </div>
          {/if}
        </div>
      {/if}
      {#if memHistory.length > 0}
        {@const s = getStats(memHistory)}
        <div class="stats">
          <span>min: {s.min}%</span>
          <span>avg: {s.avg}%</span>
          <span>max: {s.max}%</span>
        </div>
      {/if}
    </div>

    <div class="gauge">
      <div class="gauge-header">
        <span class="gauge-label">{t('tools.swap')}</span>
        <span class="gauge-value">{swap}%</span>
      </div>
      <div class="gauge-bar-container">
        <div class="gauge-bar swap" style:width="{swap}%"></div>
      </div>
    </div>

    <div class="gauge">
      <div class="gauge-header">
        <span class="gauge-label">{t('tools.networkIO')}</span>
        <span class="gauge-value">
          <span class="net-rx">&#8595; {formatRate(netRx)}</span>
          <span class="net-tx">&#8593; {formatRate(netTx)}</span>
        </span>
      </div>
      {#if netRxHistory.length > 1}
        <div class="graph-container">
          <canvas
            bind:this={netCanvas}
            class="graph-canvas"
            onmousemove={(e) => handleGraphHover(e, netRxHistory, Math.max(...netRxHistory, ...netTxHistory, 1024), 'Net RX', true)}
            onmouseleave={hideTooltip}
          ></canvas>
          {#if tooltipVisible && tooltipMetric === 'Net RX'}
            <div class="tooltip" style:left="{tooltipX}px" style:top="{tooltipY}px">
              {tooltipValue}
            </div>
          {/if}
        </div>
      {/if}
      {#if netRxHistory.length > 0}
        {@const s = getStats(netRxHistory)}
        <div class="stats">
          <span>min: {formatRate(s.min)}</span>
          <span>avg: {formatRate(s.avg)}</span>
          <span>max: {formatRate(s.max)}</span>
        </div>
      {/if}
    </div>

    <div class="gauge">
      <div class="gauge-header">
        <span class="gauge-label">{t('tools.diskIO')}</span>
        <span class="gauge-value">
          <span class="disk-read">R: {formatRate(diskRead)}</span>
          <span class="disk-write">W: {formatRate(diskWrite)}</span>
        </span>
      </div>
      {#if diskReadHistory.length > 1}
        <div class="graph-container">
          <canvas
            bind:this={diskCanvas}
            class="graph-canvas"
            onmousemove={(e) => handleGraphHover(e, diskReadHistory, Math.max(...diskReadHistory, ...diskWriteHistory, 1024), 'Disk Read', true)}
            onmouseleave={hideTooltip}
          ></canvas>
          {#if tooltipVisible && tooltipMetric === 'Disk Read'}
            <div class="tooltip" style:left="{tooltipX}px" style:top="{tooltipY}px">
              {tooltipValue}
            </div>
          {/if}
        </div>
      {/if}
      {#if diskReadHistory.length > 0}
        {@const s = getStats(diskReadHistory)}
        <div class="stats">
          <span>min: {formatRate(s.min)}</span>
          <span>avg: {formatRate(s.avg)}</span>
          <span>max: {formatRate(s.max)}</span>
        </div>
      {/if}
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
    overflow-y: auto;
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
    display: flex;
    gap: 12px;
  }

  .net-rx {
    color: #61afef;
  }

  .net-tx {
    color: #e5c07b;
  }

  .disk-read {
    color: #98c379;
  }

  .disk-write {
    color: #e06c75;
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

  .graph-container {
    position: relative;
    width: 100%;
    height: 80px;
    margin-top: 4px;
  }

  .graph-canvas {
    width: 100%;
    height: 100%;
    background: var(--bg-secondary);
    border-radius: 4px;
    cursor: crosshair;
  }

  .tooltip {
    position: absolute;
    background: var(--bg-primary);
    border: 1px solid var(--border-primary);
    border-radius: 4px;
    padding: 2px 6px;
    font-size: 11px;
    font-family: 'JetBrains Mono', monospace;
    color: var(--text-primary);
    pointer-events: none;
    transform: translateX(-50%);
    white-space: nowrap;
    z-index: 10;
  }

  .stats {
    display: flex;
    justify-content: space-between;
    font-size: 10px;
    color: var(--text-tertiary);
    font-family: 'JetBrains Mono', monospace;
    padding: 0 4px;
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
