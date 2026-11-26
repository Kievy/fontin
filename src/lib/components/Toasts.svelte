<script lang="ts">
  import { toasts, dismissToast } from "$lib/store.svelte";
</script>

<div class="toasts" aria-live="polite">
  {#each toasts as t (t.id)}
    <div class="toast {t.kind}" class:has-action={!!t.action}>
      <span class="dot"></span>
      <span class="body">
        <span>{t.text}</span>
        {#if t.detail}<span class="detail">{t.detail}</span>{/if}
      </span>
      {#if t.action}
        <button
          class="action"
          onclick={() => {
            dismissToast(t.id);
            t.action?.run();
          }}>{t.action.label}</button
        >
      {/if}
    </div>
  {/each}
</div>

<style>
  .toasts {
    position: fixed;
    bottom: var(--sp-24);
    left: 50%;
    translate: -50% 0;
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: var(--sp-8);
    z-index: 100;
    pointer-events: none;
  }
  .toast {
    display: flex;
    align-items: center;
    gap: var(--sp-12);
    max-width: 560px;
    padding: var(--sp-8) var(--sp-16);
    border-radius: var(--r-12);
    background: var(--toast-bg);
    backdrop-filter: blur(16px);
    box-shadow: var(--shadow-pop);
    font-size: 13px;
    animation: up 220ms var(--ease);
  }
  .dot {
    width: 8px;
    height: 8px;
    flex: none;
    border-radius: 50%;
    background: var(--text-3);
  }
  .success .dot {
    background: var(--mint);
    box-shadow: 0 0 8px var(--mint);
  }
  .error .dot {
    background: var(--danger);
  }
  @keyframes up {
    from {
      opacity: 0;
      transform: translateY(8px);
    }
  }
  .body {
    display: flex;
    flex-direction: column;
    min-width: 0;
  }
  .detail {
    font-size: 12px;
    line-height: 16px;
    color: var(--text-3);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    max-width: 360px;
  }
  .has-action {
    pointer-events: auto;
    padding-right: var(--sp-8);
  }
  .action {
    flex: none;
    height: 32px;
    padding: 0 var(--sp-12);
    border-radius: var(--r-8);
    background: var(--mint);
    color: var(--ink);
    font-size: 13px;
    font-weight: 600;
  }
  .action:hover {
    filter: brightness(1.05);
  }
</style>
