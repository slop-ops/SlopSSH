<script lang="ts">
  import * as api from '$lib/api/invoke'

  let { sessionId }: { sessionId: string } = $props()

  let searchDir = $state('/')
  let pattern = $state('')
  let contentFilter = $state('')
  let results = $state<string[]>([])
  let loading = $state(false)
  let error = $state('')
  let maxResults = $state(100)

  async function searchFiles() {
    if (!pattern.trim()) return
    loading = true
    error = ''
    results = []
    try {
      let cmd: string
      if (contentFilter.trim()) {
        cmd = `find ${searchDir} -name '${pattern}' -type f -exec grep -l '${contentFilter}' {} \\; 2>/dev/null | head -${maxResults}`
      } else {
        cmd = `find ${searchDir} -name '${pattern}' 2>/dev/null | head -${maxResults}`
      }
      const result = await api.remoteExec(sessionId, cmd, 60)
      results = result.stdout.split('\n').filter((l: string) => l.trim())
      if (results.length === 0 && result.exitCode !== 0) {
        error = 'Search returned no results'
      }
    } catch (e) {
      error = String(e)
    } finally {
      loading = false
    }
  }

  function searchByName() {
    contentFilter = ''
    searchFiles()
  }
</script>

<div class="search-panel">
  <div class="toolbar">
    <div class="search-fields">
      <div class="field-row">
        <label>Dir</label>
        <input
          type="text"
          bind:value={searchDir}
          class="input"
          onkeydown={(e) => { if (e.key === 'Enter') searchByName() }}
        />
      </div>
      <div class="field-row">
        <label>Name</label>
        <input
          type="text"
          bind:value={pattern}
          placeholder="*.log, *.conf..."
          class="input"
          onkeydown={(e) => { if (e.key === 'Enter') searchByName() }}
        />
      </div>
      <div class="field-row">
        <label>Content</label>
        <input
          type="text"
          bind:value={contentFilter}
          placeholder="Filter by content..."
          class="input"
          onkeydown={(e) => { if (e.key === 'Enter') searchFiles() }}
        />
      </div>
    </div>
    <div class="search-actions">
      <button class="action-btn primary" onclick={searchFiles} disabled={loading || !pattern.trim()}>
        {loading ? '...' : 'Search'}
      </button>
    </div>
  </div>

  {#if error}
    <div class="error">{error}</div>
  {/if}

  <div class="results-header">
    <span class="results-count">{results.length} results</span>
    <select bind:value={maxResults} class="limit-select">
      <option value={50}>50</option>
      <option value={100}>100</option>
      <option value={500}>500</option>
      <option value={1000}>1000</option>
    </select>
  </div>

  <div class="results">
    {#each results as path, i}
      <div class="result-item">
        <span class="result-num">{i + 1}</span>
        <span class="result-path" title={path}>{path}</span>
      </div>
    {/each}

    {#if results.length === 0 && !loading && !error}
      <div class="empty">Enter a search pattern to find files</div>
    {/if}
  </div>
</div>

<style>
  .search-panel {
    display: flex;
    flex-direction: column;
    height: 100%;
    background: #1e1f2b;
  }

  .toolbar {
    display: flex;
    gap: 12px;
    padding: 8px 12px;
    border-bottom: 1px solid #2e303a;
    align-items: flex-end;
  }

  .search-fields {
    flex: 1;
    display: flex;
    flex-direction: column;
    gap: 4px;
  }

  .field-row {
    display: flex;
    align-items: center;
    gap: 8px;
  }

  .field-row label {
    font-size: 11px;
    color: #6b7280;
    min-width: 50px;
    text-transform: uppercase;
    letter-spacing: 0.5px;
  }

  .input {
    flex: 1;
    background: #16171d;
    border: 1px solid #2e303a;
    border-radius: 4px;
    padding: 4px 8px;
    color: #e0e0e0;
    font-size: 12px;
    font-family: 'JetBrains Mono', monospace;
    outline: none;
  }

  .input:focus {
    border-color: #4a90d9;
  }

  .search-actions {
    flex-shrink: 0;
  }

  .action-btn {
    background: transparent;
    border: 1px solid #2e303a;
    color: #9ca3af;
    padding: 5px 16px;
    border-radius: 4px;
    cursor: pointer;
    font-size: 12px;
    white-space: nowrap;
  }

  .action-btn:hover {
    background: #2a2a3e;
    color: #e0e0e0;
  }

  .action-btn.primary {
    background: #4a90d922;
    border-color: #4a90d9;
    color: #4a90d9;
  }

  .action-btn.primary:hover {
    background: #4a90d944;
  }

  .action-btn:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .error {
    background: #e06c7522;
    color: #e06c75;
    padding: 6px 12px;
    font-size: 12px;
  }

  .results-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 4px 12px;
    border-bottom: 1px solid #2e303a;
  }

  .results-count {
    font-size: 11px;
    color: #6b7280;
  }

  .limit-select {
    background: #16171d;
    border: 1px solid #2e303a;
    border-radius: 3px;
    color: #9ca3af;
    font-size: 11px;
    padding: 2px 4px;
  }

  .results {
    flex: 1;
    overflow-y: auto;
  }

  .result-item {
    display: flex;
    gap: 8px;
    padding: 3px 12px;
    align-items: center;
  }

  .result-item:hover {
    background: #2a2a3e;
  }

  .result-num {
    color: #4a4a5a;
    font-size: 10px;
    min-width: 30px;
    text-align: right;
  }

  .result-path {
    color: #e0e0e0;
    font-size: 12px;
    font-family: 'JetBrains Mono', monospace;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .empty {
    text-align: center;
    color: #6b7280;
    padding: 24px;
    font-size: 13px;
  }
</style>
