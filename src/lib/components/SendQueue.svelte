<script lang="ts">
  type QueueState = 'queued' | 'cancelled';
  interface QueuedFile { id: string; file: File; state: QueueState; }

  let queuedFiles = $state<QueuedFile[]>([]);

  function selectFiles(event: Event) {
    const input = event.currentTarget as HTMLInputElement;
    const newFiles = Array.from(input.files ?? []).map((file) => ({ id: crypto.randomUUID(), file, state: 'queued' as const }));
    queuedFiles = [...queuedFiles, ...newFiles];
    input.value = '';
  }

  function cancel(id: string) {
    queuedFiles = queuedFiles.map((item) => item.id === id ? { ...item, state: 'cancelled' } : item);
  }

  function remove(id: string) { queuedFiles = queuedFiles.filter((item) => item.id !== id); }

  function size(bytes: number) { return bytes < 1024 * 1024 ? `${Math.ceil(bytes / 1024)} KB` : `${(bytes / 1024 / 1024).toFixed(1)} MB`; }
</script>

<section aria-labelledby="send-heading">
  <p class="eyebrow">Send files</p>
  <h2 id="send-heading">Choose files for a paired device.</h2>
  <p>Files remain queued until you select a paired peer. Transfers report verified byte progress once a secure session is connected.</p>
  <label class="picker" for="file-picker">Choose files</label>
  <input id="file-picker" type="file" multiple onchange={selectFiles} />

  {#if queuedFiles.length}
    <ul aria-label="Transfer queue">
      {#each queuedFiles as item (item.id)}
        <li>
          <div><strong>{item.file.name}</strong><span>{size(item.file.size)} · {item.state}</span></div>
          {#if item.state === 'queued'}
            <progress value="0" max={item.file.size} aria-label={`${item.file.name} queued`} />
            <button class="secondary" onclick={() => cancel(item.id)}>Cancel</button>
          {:else}
            <button class="secondary" onclick={() => remove(item.id)}>Remove</button>
          {/if}
        </li>
      {/each}
    </ul>
  {/if}
</section>

<style>
  section { max-width: 36rem; margin-top: 2rem; padding: 1.5rem; border: 1px solid rgba(201,236,227,.2); border-radius: 1rem; background: rgba(8,31,33,.7); }
  h2 { margin: .25rem 0; font-size: 1.5rem; } p, span { color: #bdd2cf; } .eyebrow { color: #7df0cb; font-size: .8rem; font-weight: 700; letter-spacing: .1em; text-transform: uppercase; }
  input { position: absolute; inline-size: 1px; block-size: 1px; opacity: 0; } .picker, button { display: inline-block; margin-top: .75rem; padding: .7rem 1rem; border: 0; border-radius: .6rem; color: #06221d; background: #7df0cb; font: inherit; font-weight: 700; cursor: pointer; }
  .secondary { margin: 0; color: white; background: #28504d; } .picker:focus-visible, button:focus-visible { outline: 3px solid white; outline-offset: 3px; }
  ul { padding: 0; list-style: none; } li { display: grid; grid-template-columns: 1fr auto; gap: .75rem; align-items: center; padding: 1rem 0; border-top: 1px solid rgba(201,236,227,.16); } span { display: block; font-size: .9rem; } progress { width: 100%; grid-column: 1; }
</style>
