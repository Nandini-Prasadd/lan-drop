<script lang="ts">
  import { invoke } from '@tauri-apps/api/core';
  import type { TransferMetadata } from '$lib/models/transfer';
  interface HistoryRecord { metadata: TransferMetadata; failureMessage: string | null; recordedAt: string; }
  let records = $state<HistoryRecord[]>([]); let error = $state<string | null>(null);
  async function refresh() { try { records = await invoke<HistoryRecord[]>('list_transfer_history'); error = null; } catch { error = 'Could not load local transfer history.'; } }
  refresh();
</script>
<section aria-labelledby="history-heading"><p class="eyebrow">History</p><h2 id="history-heading">Recent local transfers</h2><button onclick={refresh}>Refresh</button>
{#if error}<p role="alert">{error}</p>{:else if !records.length}<p>No transfers have been recorded yet.</p>{:else}<ul>{#each records as record (record.metadata.id)}<li><strong>{record.metadata.file.name}</strong><span>{record.metadata.state} · {record.metadata.peer.displayName}</span>{#if record.failureMessage}<p role="alert">{record.failureMessage}</p>{/if}</li>{/each}</ul>{/if}</section>
<style>section{max-width:36rem;margin-top:2rem;padding:1.5rem;border:1px solid rgba(201,236,227,.2);border-radius:1rem;background:rgba(8,31,33,.7)}h2{margin:.25rem 0}p,span{color:#bdd2cf}.eyebrow{color:#7df0cb;font-size:.8rem;font-weight:700;letter-spacing:.1em;text-transform:uppercase}button{padding:.6rem .9rem;border:0;border-radius:.6rem;color:#06221d;background:#7df0cb;font:inherit;font-weight:700}li{padding:1rem 0;border-top:1px solid rgba(201,236,227,.16)}span{display:block}</style>
