<script lang="ts">
  import Sidebar from './Sidebar.svelte'
  import Terminal from '$lib/terminal/Terminal.svelte'

  let showSidebar = $state(true)
  let activeSessionId = $state('')
  let activeChannelId = $state('')
</script>

<div class="app-shell">
  {#if showSidebar}
    <aside class="sidebar">
      <Sidebar
        onConnect={(id: string) => {
          activeSessionId = id
          activeChannelId = crypto.randomUUID()
        }}
      />
    </aside>
  {/if}
  <main class="content">
    {#if activeSessionId}
      {#key activeChannelId}
        <Terminal sessionId={activeSessionId} channelId={activeChannelId} />
      {/key}
    {:else}
      <div class="welcome">
        <h1>Muon SSH</h1>
        <p>Select a session to connect</p>
      </div>
    {/if}
  </main>
</div>

<style>
  .app-shell {
    display: flex;
    height: 100vh;
    overflow: hidden;
  }

  .sidebar {
    width: 260px;
    min-width: 200px;
    border-right: 1px solid #2e303a;
    overflow-y: auto;
    background: #16171d;
  }

  .content {
    flex: 1;
    overflow: hidden;
    background: #1a1a2e;
  }

  .welcome {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    height: 100%;
    color: #9ca3af;
  }

  .welcome h1 {
    font-size: 2em;
    color: #e0e0e0;
    margin: 0;
  }

  .welcome p {
    margin-top: 0.5em;
  }
</style>
