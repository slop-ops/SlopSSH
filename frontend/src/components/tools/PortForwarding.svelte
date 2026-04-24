<script lang="ts">
  import { portForwardStart, portForwardStop, portForwardList } from '$lib/api/invoke'

  let forwards: string[] = $state([])
  let bindHost = $state('127.0.0.1')
  let bindPort = $state(8080)
  let targetHost = $state('')
  let targetPort = $state(80)
  let direction = $state<'local' | 'remote'>('local')
  let error = $state('')
  let success = $state('')

  async function refresh() {
    try {
      forwards = await portForwardList()
    } catch (e: any) {
      error = e.toString()
    }
  }

  async function startForward() {
    error = ''
    success = ''
    if (!targetHost) {
      error = 'Target host is required'
      return
    }
    try {
      await portForwardStart(
        sessionId,
        bindHost,
        bindPort,
        targetHost,
        targetPort,
        direction,
      )
      success = `${direction} forward started on ${bindHost}:${bindPort} -> ${targetHost}:${targetPort}`
      await refresh()
      bindPort = bindPort + 1
    } catch (e: any) {
      error = e.toString()
    }
  }

  async function stopForward(id: string) {
    try {
      await portForwardStop(id)
      await refresh()
    } catch (e: any) {
      error = e.toString()
    }
  }

  let { sessionId }: { sessionId: string } = $props()

  $effect(() => {
    refresh()
  })
</script>

<div class="port-forward-panel">
  <h3>Port Forwarding</h3>

  {#if error}
    <div class="error">{error}</div>
  {/if}
  {#if success}
    <div class="success">{success}</div>
  {/if}

  <div class="forward-form">
    <div class="form-row">
      <label>
        Direction
        <select bind:value={direction}>
          <option value="local">Local (-L)</option>
          <option value="remote">Remote (-R)</option>
        </select>
      </label>
    </div>

    <div class="form-row">
      <label>
        Bind Host
        <input type="text" bind:value={bindHost} placeholder="127.0.0.1" />
      </label>
      <label>
        Bind Port
        <input type="number" bind:value={bindPort} />
      </label>
    </div>

    <div class="arrow">&#8594;</div>

    <div class="form-row">
      <label>
        Target Host
        <input type="text" bind:value={targetHost} placeholder="localhost" />
      </label>
      <label>
        Target Port
        <input type="number" bind:value={targetPort} />
      </label>
    </div>

    <button class="btn-primary" onclick={startForward}>Start Forward</button>
  </div>

  <div class="active-forwards">
    <h4>Active Forwards ({forwards.length})</h4>
    {#if forwards.length === 0}
      <p class="empty">No active port forwards</p>
    {:else}
      {#each forwards as id}
        <div class="forward-item">
          <span class="forward-id">{id}</span>
          <button class="btn-danger btn-small" onclick={() => stopForward(id)}>Stop</button>
        </div>
      {/each}
    {/if}
  </div>
</div>

<style>
  .port-forward-panel {
    padding: 12px;
  }
  h3 {
    margin: 0 0 12px 0;
    color: var(--text-primary);
  }
  .forward-form {
    display: flex;
    flex-direction: column;
    gap: 8px;
    padding: 12px;
    background: var(--bg-secondary);
    border-radius: 6px;
    margin-bottom: 16px;
  }
  .form-row {
    display: flex;
    gap: 12px;
    align-items: center;
  }
  label {
    display: flex;
    flex-direction: column;
    gap: 4px;
    font-size: 12px;
    color: var(--text-secondary);
    flex: 1;
  }
  input, select {
    padding: 6px 8px;
    border: 1px solid var(--border);
    border-radius: 4px;
    background: var(--bg-primary);
    color: var(--text-primary);
    font-size: 13px;
  }
  .arrow {
    text-align: center;
    color: var(--text-secondary);
    font-size: 18px;
  }
  .btn-primary {
    padding: 8px 16px;
    background: var(--accent);
    color: white;
    border: none;
    border-radius: 4px;
    cursor: pointer;
    font-size: 13px;
  }
  .btn-primary:hover {
    opacity: 0.9;
  }
  .btn-danger {
    background: var(--error);
  }
  .btn-small {
    padding: 4px 8px;
    font-size: 11px;
  }
  .active-forwards h4 {
    margin: 0 0 8px 0;
    color: var(--text-primary);
  }
  .empty {
    color: var(--text-secondary);
    font-size: 13px;
  }
  .forward-item {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 6px 8px;
    background: var(--bg-secondary);
    border-radius: 4px;
    margin-bottom: 4px;
  }
  .forward-id {
    font-family: monospace;
    font-size: 12px;
    color: var(--text-primary);
  }
  .error {
    color: var(--error);
    font-size: 13px;
    padding: 8px;
    background: rgba(255, 0, 0, 0.1);
    border-radius: 4px;
    margin-bottom: 8px;
  }
  .success {
    color: #4caf50;
    font-size: 13px;
    padding: 8px;
    background: rgba(76, 175, 80, 0.1);
    border-radius: 4px;
    margin-bottom: 8px;
  }
</style>
