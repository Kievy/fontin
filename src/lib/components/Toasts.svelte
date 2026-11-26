<script lang="ts">
  import { toasts } from "$lib/store.svelte";
</script>

<div class="toasts" aria-live="polite">
  {#each toasts as t (t.id)}
    <div class="toast {t.kind}">
      <span class="dot"></span>
      {t.text}
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
</style>
