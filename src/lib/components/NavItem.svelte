<script lang="ts">
  import type { Snippet } from "svelte";
  import type { FamilyRow } from "$lib/api";
  import { drag } from "$lib/store.svelte";
  let {
    active = false,
    count = null,
    depth = 0,
    icon,
    actions,
    trailing,
    children,
    onclick,
    ondropfamily,
  }: {
    active?: boolean;
    count?: number | null;
    depth?: number;
    icon?: Snippet;
    actions?: Snippet;
    /** Always visible after the count/actions (e.g. a toggle). */
    trailing?: Snippet;
    children: Snippet;
    onclick?: () => void;
    /** Makes the item a drop target for families dragged from the list. */
    ondropfamily?: (row: FamilyRow) => void;
  } = $props();

  // dragenter/leave fire for every child; a counter keeps the highlight stable.
  let depthIn = $state(0);
  const accepting = $derived(!!ondropfamily && !!drag.row);
</script>

<div
  class="nav"
  class:active
  class:target={accepting}
  class:over={accepting && depthIn > 0}
  style="--depth: {depth}"
  role="listitem"
  ondragenter={(e) => {
    if (!accepting) return;
    e.preventDefault();
    depthIn++;
  }}
  ondragover={(e) => {
    if (!accepting) return;
    e.preventDefault();
    if (e.dataTransfer) e.dataTransfer.dropEffect = "copy";
  }}
  ondragleave={() => {
    if (accepting) depthIn = Math.max(0, depthIn - 1);
  }}
  ondrop={(e) => {
    if (!accepting || !drag.row) return;
    e.preventDefault();
    depthIn = 0;
    ondropfamily?.(drag.row);
  }}
>
  <button class="main" {onclick}>
    {#if icon}<span class="icon">{@render icon()}</span>{/if}
    <span class="label">{@render children()}</span>
    {#if count !== null}<span class="count tabular">{count.toLocaleString("pt-BR")}</span>{/if}
  </button>
  {#if actions}<span class="actions">{@render actions()}</span>{/if}
  {#if trailing}<span class="trailing">{@render trailing()}</span>{/if}
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
  .nav.target {
    box-shadow: inset 0 0 0 1px var(--line-2);
    border-radius: var(--r-8);
  }
  .nav.over {
    background: var(--mint-soft);
    color: var(--text);
    box-shadow: inset 0 0 0 1px var(--accent-line);
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
    color: var(--accent-text);
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
  .trailing {
    display: flex;
    padding-right: var(--sp-8);
  }
  .nav:has(.trailing) .main {
    padding-right: var(--sp-8);
  }
  .nav:has(.trailing):hover .actions {
    padding-right: var(--sp-4);
  }
  .nav:has(.actions):hover .main {
    padding-right: 0;
  }
</style>
