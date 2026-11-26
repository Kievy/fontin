<script lang="ts">
  import Heart from "@lucide/svelte/icons/heart";
  import Toggle from "./Toggle.svelte";
  import type { FamilyRow } from "$lib/api";
  import { loadRow, rowFamily } from "$lib/fonts";
  import { data, prefs, ui, drag, select, toggleFamily, toggleFavorite } from "$lib/store.svelte";

  let { row, mode, height }: { row: FamilyRow; mode: "list" | "grid"; height: number } = $props();

  let state = $state<"loading" | "ready" | "failed">("loading");

  $effect(() => {
    const r = row;
    state = "loading";
    let alive = true;
    loadRow(r).then((ok) => alive && (state = ok ? "ready" : "failed"));
    return () => (alive = false);
  });

  const family = $derived(rowFamily(row));

  function onDragStart(e: DragEvent) {
    if (!e.dataTransfer) return;
    drag.row = row;
    e.dataTransfer.effectAllowed = "copy";
    e.dataTransfer.setData("text/plain", row.name);
    // Compact pill instead of a screenshot of the whole row.
    const ghost = document.createElement("div");
    ghost.className = "drag-ghost";
    ghost.textContent = row.name;
    if (family) ghost.style.fontFamily = `'${family}', var(--font-ui)`;
    document.body.appendChild(ghost);
    e.dataTransfer.setDragImage(ghost, 16, 20);
    requestAnimationFrame(() => ghost.remove());
  }
  const dl = $derived(data.downloads[row.id]);
  const size = $derived(mode === "grid" ? Math.min(prefs.size, 56) : prefs.size);
  const category = $derived(
    row.category &&
      ({ "Sans Serif": "Sans", Serif: "Serif", Display: "Display", Handwriting: "Manuscrita", Monospace: "Mono" }[
        row.category
      ] ??
        row.category),
  );
</script>

<div
  class="row {mode}"
  class:selected={ui.selected === row.id}
  class:active={row.active > 0}
  class:dragging={drag.row?.id === row.id}
  style="height: {height}px"
  draggable="true"
  ondragstart={onDragStart}
  ondragend={() => (drag.row = null)}
  role="button"
  tabindex="-1"
  onclick={() => select(row.id)}
  onkeydown={(e) => e.key === "Enter" && select(row.id)}
>
  <div class="meta">
    <Toggle
      state={row.active}
      busy={!!data.pending[row.id]}
      progress={dl ? (dl.total ? dl.done / dl.total : 0) : null}
      label={row.active === 2 ? `Desativar ${row.name}` : `Ativar ${row.name}`}
      onchange={() => toggleFamily(row)}
    />
    <span class="name">{row.name}</span>
    <span class="chip tabular">{row.styles} {row.styles === 1 ? "estilo" : "estilos"}</span>
    {#if row.provider === "google"}
      <span class="chip google">{category ?? "Google"}</span>
    {:else}
      <span class="chip">{row.format}</span>
    {/if}
    {#if dl}
      <span class="dl tabular">baixando {dl.done}/{dl.total}</span>
    {/if}
    <button
      class="fav"
      class:on={row.favorite}
      aria-label={row.favorite ? "Remover dos favoritos" : "Favoritar"}
      onclick={(e) => {
        e.stopPropagation();
        toggleFavorite(row);
      }}
    >
      <Heart size={16} strokeWidth={1.75} fill={row.favorite ? "currentColor" : "none"} />
    </button>
  </div>

  <div
    class="preview"
    class:ready={state === "ready"}
    class:failed={state === "failed"}
    style="font-family: {family ? `'${family}', ` : ''}var(--font-ui); font-size: {size}px;"
  >
    {state === "failed" ? "Prévia indisponível" : prefs.previewText || row.name}
  </div>
</div>

<style>
  .row {
    position: relative;
    display: flex;
    flex-direction: column;
    gap: var(--sp-8);
    padding: var(--sp-16) var(--sp-24);
    border-radius: var(--r-16);
    cursor: default;
    transition: background 140ms var(--ease);
    overflow: hidden;
  }
  .row.dragging {
    opacity: 0.4;
  }
  .row:hover {
    background: var(--row-hover);
  }
  .row.selected {
    background: var(--raised);
    box-shadow: inset 0 0 0 1px var(--line-2);
  }
  .row.grid {
    background: var(--raised);
    box-shadow: inset 0 0 0 1px var(--line);
    padding: var(--sp-16);
  }
  .row.grid:hover {
    box-shadow: inset 0 0 0 1px var(--line-2);
  }
  .row.grid.selected {
    box-shadow: inset 0 0 0 1px var(--accent-line);
  }
  .meta {
    display: flex;
    align-items: center;
    gap: var(--sp-12);
    height: 24px;
    min-width: 0;
  }
  .name {
    font-weight: 600;
    font-size: 13px;
    color: var(--text);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    min-width: 0;
  }
  .row:not(.active) .name {
    color: var(--text-2);
  }
  .chip {
    flex: none;
    height: 20px;
    padding: 0 var(--sp-8);
    display: inline-flex;
    align-items: center;
    border-radius: var(--r-full);
    background: var(--raised-2);
    color: var(--text-3);
    font-size: 11px;
    font-weight: 500;
    letter-spacing: 0.02em;
  }
  .grid .chip:not(.google) {
    display: none;
  }
  .chip.google {
    background: var(--info-bg);
    color: var(--info-text);
  }
  .dl {
    font-size: 12px;
    color: var(--accent-text);
  }
  .fav {
    margin-left: auto;
    flex: none;
    width: 32px;
    height: 32px;
    display: grid;
    place-items: center;
    border-radius: var(--r-8);
    color: var(--text-3);
    opacity: 0;
    transition:
      opacity 120ms var(--ease),
      color 120ms var(--ease);
  }
  .row:hover .fav,
  .fav.on {
    opacity: 1;
  }
  .fav:hover {
    color: var(--text);
    background: var(--press);
  }
  .fav.on {
    color: var(--heart);
  }
  .preview {
    flex: 1;
    min-height: 0;
    line-height: 1.3;
    color: var(--text);
    white-space: nowrap;
    overflow: hidden;
    font-synthesis: none;
    opacity: 0;
    transform: translateY(4px);
    transition:
      opacity 220ms var(--ease),
      transform 220ms var(--ease);
    -webkit-mask-image: linear-gradient(to right, #000 calc(100% - 64px), transparent);
    mask-image: linear-gradient(to right, #000 calc(100% - 64px), transparent);
  }
  .grid .preview {
    white-space: normal;
    display: -webkit-box;
    -webkit-line-clamp: 2;
    line-clamp: 2;
    -webkit-box-orient: vertical;
    -webkit-mask-image: none;
    mask-image: none;
    word-break: break-word;
  }
  .row:not(.active) .preview {
    color: var(--text-2);
  }
  .preview.ready {
    opacity: 1;
    transform: none;
  }
  .preview.failed {
    opacity: 1;
    transform: none;
    color: var(--text-4) !important;
    font-size: 14px !important;
  }
</style>
