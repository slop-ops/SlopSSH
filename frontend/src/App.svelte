<script lang="ts">
  import { invoke } from '@tauri-apps/api/core'

  let greeting = $state('')
  let name = $state('')

  async function greet() {
    greeting = await invoke<string>('greet', { name })
  }
</script>

<main>
  <div class="logo">
    <h1>Muon SSH</h1>
  </div>
  <form onsubmit={(e) => { e.preventDefault(); greet() }}>
    <input bind:value={name} placeholder="Enter a name..." />
    <button type="submit">Greet</button>
  </form>
  {#if greeting}
    <p>{greeting}</p>
  {/if}
</main>
