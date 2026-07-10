<script lang="ts">
  import { invoke } from '@tauri-apps/api/core';

  interface PairingInvitation {
    code: string;
    expiresAtEpochSeconds: number;
    deviceName: string;
    publicKey: string;
    qrSvg: string;
  }

  let invitation = $state<PairingInvitation | null>(null);
  let error = $state<string | null>(null);
  let isCreating = $state(false);
  let manualAddress = $state('');
  let manualStatus = $state<string | null>(null);

  async function createInvitation() {
    isCreating = true;
    error = null;
    try {
      invitation = await invoke<PairingInvitation>('create_pairing_invitation');
    } catch {
      error = 'Could not create a local pairing invitation. Try again.';
    } finally {
      isCreating = false;
    }
  }

  function qrImage(svg: string) {
    return `data:image/svg+xml;charset=utf-8,${encodeURIComponent(svg)}`;
  }

  async function validateManualAddress() {
    try {
      const peer = await invoke<{ address: string }>('validate_manual_peer_address', { address: manualAddress });
      manualStatus = `Ready to pair with ${peer.address}.`;
    } catch {
      manualStatus = 'Enter a valid address with a port, such as 192.168.1.20:47473.';
    }
  }
</script>

<section aria-labelledby="pair-heading">
  <div>
    <p class="eyebrow">Pair a device</p>
    <h2 id="pair-heading">Share a QR code or short code.</h2>
    <p>Pairing invitations expire after five minutes and contain no file data or private key.</p>
  </div>
  <button onclick={createInvitation} disabled={isCreating}>
    {isCreating ? 'Creating invitation…' : 'Create pairing invitation'}
  </button>

  {#if invitation}
    <div class="invitation" aria-live="polite">
      <img src={qrImage(invitation.qrSvg)} alt={`Pair with ${invitation.deviceName} using this QR code`} />
      <p>Short code: <strong>{invitation.code}</strong></p>
      <p class="muted">Expires at {new Date(invitation.expiresAtEpochSeconds * 1000).toLocaleTimeString()}.</p>
    </div>
  {:else if error}
    <p class="error" role="alert">{error}</p>
  {/if}

  <form onsubmit={(event) => { event.preventDefault(); validateManualAddress(); }}>
    <label for="peer-address">Pair by address when discovery is unavailable</label>
    <div class="manual-entry">
      <input id="peer-address" bind:value={manualAddress} placeholder="192.168.1.20:47473" autocomplete="off" />
      <button type="submit">Validate address</button>
    </div>
    {#if manualStatus}<p class="muted" aria-live="polite">{manualStatus}</p>{/if}
  </form>
</section>

<style>
  section { max-width: 36rem; margin-top: 3rem; padding: 1.5rem; border: 1px solid rgba(201,236,227,.2); border-radius: 1rem; background: rgba(8,31,33,.7); }
  h2 { margin: .25rem 0; font-size: 1.5rem; }
  p { color: #bdd2cf; }
  .eyebrow { color: #7df0cb; font-size: .8rem; font-weight: 700; letter-spacing: .1em; text-transform: uppercase; }
  button { padding: .7rem 1rem; border: 0; border-radius: .6rem; color: #06221d; background: #7df0cb; font: inherit; font-weight: 700; cursor: pointer; }
  button:focus-visible { outline: 3px solid white; outline-offset: 3px; }
  button:disabled { cursor: wait; opacity: .7; }
  .invitation { display: grid; grid-template-columns: 9rem 1fr; gap: 1rem; align-items: center; margin-top: 1.5rem; }
  img { width: 9rem; height: 9rem; padding: .5rem; background: white; border-radius: .5rem; }
  strong { color: white; font-family: ui-monospace, monospace; font-size: 1.3rem; letter-spacing: .08em; }
  .muted { font-size: .9rem; }
  .error { color: #ffded8; }
  form { margin-top: 1.5rem; }
  label { display: block; margin-bottom: .5rem; color: white; font-weight: 650; }
  .manual-entry { display: flex; gap: .5rem; }
  input { min-width: 0; flex: 1; padding: .65rem .75rem; border: 1px solid #6d8d87; border-radius: .6rem; color: white; background: #102b2a; font: inherit; }
  input:focus-visible { outline: 3px solid white; outline-offset: 2px; }
  @media (max-width: 32rem) { .invitation { grid-template-columns: 1fr; } }
</style>
