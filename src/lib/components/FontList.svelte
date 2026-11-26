<script lang="ts">
  import FontRow from "./FontRow.svelte";
  import Empty from "./Empty.svelte";
  import { data, prefs, ui } from "$lib/store.svelte";

  const PAD = 24;
  const GAP = 8;
  const CARD_MIN = 280;
  const OVERSCAN = 4;

  let scrollTop = $state(0);
  let viewH = $state(0);
  let viewW = $state(0);
  let el: HTMLDivElement | undefined = $state();

  const ceil8 = (n: number) => Math.ceil(n / 8) * 8;

  const cols = $derived(
    prefs.view === "grid" ? Math.max(1, Math.floor((viewW - PAD * 2 + GAP) / (CARD_MIN + GAP))) : 1,
  );
  const itemH = $derived(
    prefs.view === "grid"
      ? 16 + 24 + 8 + ceil8(Math.min(prefs.size, 56) * 1.3) * 2 + 16
      : 16 + 24 + 8 + ceil8(prefs.size * 1.3) + 16,
  );
  const lineH = $derived(itemH + GAP);
  const lines = $derived(Math.ceil(data.rows.length / cols));
  const first = $derived(Math.max(0, Math.floor((scrollTop - PAD) / lineH) - OVERSCAN));
  const last = $derived(Math.min(lines, Math.ceil((scrollTop + viewH) / lineH) + OVERSCAN));
  const cardW = $derived((viewW - PAD * 2 - GAP * (cols - 1)) / cols);

  const visible = $derived.by(() => {
    const out = [];
    for (let line = first; line < last; line++) {
      for (let c = 0; c < cols; c++) {
        const i = line * cols + c;
        if (i >= data.rows.length) break;
        out.push({ row: data.rows[i], top: PAD + line * lineH, left: PAD + c * (cardW + GAP) });
      }
    }
    return out;
  });

  // Jump back to the top whenever the result set changes context.
  $effect(() => {
    void ui.scope;
    void ui.category;
    void ui.sort;
    if (el) el.scrollTop = 0;
  });
</script>

<div
  class="list"
  bind:this={el}
  bind:clientHeight={viewH}
  bind:clientWidth={viewW}
  onscroll={(e) => (scrollTop = (e.target as HTMLDivElement).scrollTop)}
>
  {#if !data.loading && data.rows.length === 0}
    <Empty />
  {:else}
    <div class="spacer" style="height: {PAD * 2 + lines * lineH}px">
      {#each visible as v (v.row.id)}
        <div class="slot" style="transform: translate({v.left}px, {v.top}px); width: {cardW}px">
          <FontRow row={v.row} mode={prefs.view} height={itemH} />
        </div>
      {/each}
    </div>
  {/if}
</div>

<style>
  .list {
    flex: 1;
    min-height: 0;
    overflow-y: auto;
    overflow-x: hidden;
    position: relative;
    contain: strict;
  }
  .spacer {
    position: relative;
  }
  .slot {
    position: absolute;
    top: 0;
    left: 0;
    will-change: transform;
  }
</style>
