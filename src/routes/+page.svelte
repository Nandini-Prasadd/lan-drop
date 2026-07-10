<script lang="ts">
  import { invoke } from '@tauri-apps/api/core';
  import type { AppInfo } from '$lib/models/app';
  import PairingPanel from '$lib/components/PairingPanel.svelte';

  let appInfo = $state<AppInfo | null>(null);
  let startupError = $state<string | null>(null);

  async function loadAppInfo() {
    try {
      appInfo = await invoke<AppInfo>('app_info');
    } catch {
      startupError = 'lan-drop could not load its local application details.';
    }
  }

  loadAppInfo();
</script>

<svelte:head>
  <title>lan-drop</title>
  <meta name="description" content="Private, encrypted local-network file sharing." />
</svelte:head>

<main>
  <p class="eyebrow">Private local sharing</p>
  <h1>lan-drop is ready to pair nearby devices.</h1>
  <p class="intro">
    Files will move directly between paired devices on your local network. There is no cloud account,
    relay service, or telemetry.
  </p>

  {#if appInfo}
    <dl aria-label="Application status">
      <div><dt>Version</dt><dd>{appInfo.version}</dd></div>
      <div><dt>Data handling</dt><dd>{appInfo.storageScope}</dd></div>
    </dl>
  {:else if startupError}
    <p class="error" role="alert">{startupError}</p>
  {:else}
    <p aria-live="polite">Loading local application details…</p>
  {/if}

  <PairingPanel />
</main>

<style>
  :global(*) {
    box-sizing: border-box;
  }

  :global(body) {
    margin: 0;
    min-width: 320px;
    color: #eaf4f2;
    background:
      radial-gradient(circle at 10% 10%, rgba(89, 221, 186, 0.18), transparent 35rem),
      linear-gradient(145deg, #07181b 0%, #10282a 100%);
    font-family: Inter, ui-sans-serif, system-ui, -apple-system, BlinkMacSystemFont, "Segoe UI", sans-serif;
    line-height: 1.5;
  }

  main {
    display: grid;
    align-content: center;
    min-height: 100vh;
    max-width: 62rem;
    margin: 0 auto;
    padding: clamp(2rem, 8vw, 6rem) clamp(1.5rem, 6vw, 5rem);
  }

  .eyebrow {
    margin: 0 0 0.9rem;
    color: #7df0cb;
    font-size: 0.85rem;
    font-weight: 700;
    letter-spacing: 0.12em;
    text-transform: uppercase;
  }

  h1 {
    max-width: 18ch;
    margin: 0;
    font-size: clamp(2.3rem, 7vw, 5.5rem);
    line-height: 1;
    letter-spacing: -0.055em;
  }

  .intro {
    max-width: 58ch;
    margin: 1.8rem 0 2.25rem;
    color: #bdd2cf;
    font-size: clamp(1rem, 2vw, 1.25rem);
  }

  dl {
    display: grid;
    grid-template-columns: repeat(2, minmax(0, 1fr));
    gap: 1px;
    overflow: hidden;
    max-width: 36rem;
    margin: 0;
    border: 1px solid rgba(201, 236, 227, 0.2);
    border-radius: 1rem;
    background: rgba(201, 236, 227, 0.14);
  }

  dl div {
    padding: 1rem 1.2rem;
    background: rgba(8, 31, 33, 0.88);
  }

  dt {
    color: #9fb9b5;
    font-size: 0.8rem;
    font-weight: 700;
    letter-spacing: 0.08em;
    text-transform: uppercase;
  }

  dd {
    margin: 0.25rem 0 0;
    color: #ffffff;
    font-weight: 650;
  }

  .error {
    max-width: 48rem;
    margin: 0;
    padding: 1rem 1.2rem;
    border: 1px solid #ffb4a8;
    border-radius: 0.75rem;
    color: #ffded8;
    background: rgba(137, 30, 24, 0.32);
  }

  @media (max-width: 32rem) {
    dl {
      grid-template-columns: 1fr;
    }
  }
</style>
