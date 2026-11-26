<script lang="ts">
  import type { Snippet } from "svelte";
  let {
    active = false,
    count = null,
    depth = 0,
    icon,
    actions,
    children,
    onclick,
  }: {
    active?: boolean;
    count?: number | null;
    depth?: number;
    icon?: Snippet;
    actions?: Snippet;
    children: Snippet;
    onclick?: () => void;
  } = $props();
</script>

<div class="nav" class:active style="--depth: {depth}">
  <button class="main" {onclick}>
    {#if icon}<span class="icon">{@render icon()}</span>{/if}
    <span class="label">{@render children()}</span>
    {#if count !== null}<span class="count tabular">{count.toLocaleString("pt-BR")}</span>{/if}
  </button>
  {#if actions}<span class="actions">{@render actions()}</span>{/if}
</div>

<style>
  .nav {
    position: relative;
    display: flex;
    align-items: center;
    height: 32px;
    border-radius: var(--r-8);
    color: var(--text-2);
    transition: background 120ms var(--ease);
  }
  .nav:hover {
    background: var(--hover);
    color: var(--text);
  }
  .nav.active {
    background: var(--raised-2);
    color: var(--text);
  }
  .nav.active::before {
    content: "";
    position: absolute;
    left: 0;
    top: 8px;
    bottom: 8px;
    width: 2px;
    border-radius: 2px;
    background: var(--mint);
  }
  .main {
    flex: 1;
    min-width: 0;
    height: 100%;
    display: flex;
    align-items: center;
    gap: var(--sp-8);
    padding: 0 var(--sp-12) 0 calc(var(--sp-12) + var(--depth) * var(--sp-16));
    text-align: left;
  }
  .icon {
    display: grid;
    place-items: center;
    width: 16px;
    flex: none;
    color: var(--text-3);
  }
  .active .icon {
    color: var(--mint);
  }
  .label {
    flex: 1;
    min-width: 0;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .count {
    font-size: 12px;
    color: var(--text-3);
  }
  .actions {
    display: none;
    gap: var(--sp-4);
    padding-right: var(--sp-8);
  }
  .nav:hover .actions {
    display: flex;
  }
  .nav:hover .count {
    display: none;
  }
  .nav:has(.actions):hover .main {
    padding-right: 0;
  }
</style>
