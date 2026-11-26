<script lang="ts">
  import Search from "@lucide/svelte/icons/search";
  import X from "@lucide/svelte/icons/x";
  import List from "@lucide/svelte/icons/rows-3";
  import Grid from "@lucide/svelte/icons/layout-grid";
  import Reset from "@lucide/svelte/icons/rotate-ccw";
  import Type from "@lucide/svelte/icons/type";
  import { data, ui, prefs, savePrefs, refreshRows } from "$lib/store.svelte";
  import type { Sort } from "$lib/api";
  import { t, tn } from "$lib/i18n.svelte";

  let { searchEl = $bindable() }: { searchEl?: HTMLInputElement } = $props();

  const title = $derived.by(() => {
    const s = ui.scope;
    const o = data.overview;
    switch (s.kind) {
      case "all":
        return t("title.all");
      case "recent":
        return t("scope.recent");
      case "favorites":
        return t("scope.favorites");
      case "active":
        return t("scope.active");
      case "inactive":
        return t("scope.inactive");
      case "google":
        return t("nav.google");
      case "collection":
        return o?.collections.find((c) => c.id === s.value)?.name ?? t("title.collection");
      case "source": {
        if (s.value === 1) return t("title.googleCache");
        const src = o?.sources.find((x) => x.id === s.value);
        if (src?.kind === "files") return t("side.looseFiles");
        return src?.name ?? t("title.folder");
      }
      case "folder":
        return s.value.split(/[\\/]/).pop() ?? t("title.folder");
    }
  });

  const sorts = $derived<{ id: Sort; label: string }[]>(
    ui.section === "google"
      ? [
          { id: "popular", label: t("sort.popular") },
          { id: "name", label: t("sort.name") },
          { id: "styles", label: t("sort.styles") },
        ]
      : [
          { id: "name", label: t("sort.name") },
          { id: "recent", label: t("sort.recent") },
          { id: "styles", label: t("sort.styles") },
        ],
  );

  let searchTimer: ReturnType<typeof setTimeout> | undefined;
  function onSearch() {
    clearTimeout(searchTimer);
    searchTimer = setTimeout(refreshRows, 60);
  }
</script>

<div class="header">
  <div class="top">
    <div class="title">
      <h1>{title}</h1>
      <span class="sub tabular">
        {tn("families", data.rows.length)}
      </span>
    </div>
    <label class="search">
      <Search size={16} strokeWidth={1.75} />
      <input
        bind:this={searchEl}
        bind:value={ui.search}
        oninput={onSearch}
        placeholder={t("search.placeholder")}
        spellcheck="false"
        onkeydown={(e) => {
          if (e.key === "Escape") {
            ui.search = "";
            refreshRows();
            (e.target as HTMLInputElement).blur();
          }
        }}
      />
      {#if ui.search}
        <button
          class="clear"
          aria-label={t("search.clear")}
          onclick={() => {
            ui.search = "";
            refreshRows();
          }}><X size={14} /></button
        >
      {:else}
        <kbd>/</kbd>
      {/if}
    </label>
  </div>

  <div class="tools">
    <div class="segmented" role="group" aria-label={t("view.label")}>
      <button class:on={prefs.view === "list"} aria-label={t("view.list")} onclick={() => ((prefs.view = "list"), savePrefs())}>
        <List size={16} strokeWidth={1.75} />
      </button>
      <button class:on={prefs.view === "grid"} aria-label={t("view.grid")} onclick={() => ((prefs.view = "grid"), savePrefs())}>
        <Grid size={16} strokeWidth={1.75} />
      </button>
    </div>

    <label class="sample">
      <Type size={16} strokeWidth={1.75} />
      <input
        value={prefs.previewText || t("preview.default")}
        oninput={(e) => {
          const v = (e.target as HTMLInputElement).value;
          // Typing the language default back counts as "not customized".
          prefs.previewText = v === t("preview.default") ? "" : v;
          savePrefs();
        }}
        placeholder={t("sample.placeholder")}
      />
    </label>

    <div class="size">
      <input
        type="range"
        min="12"
        max="160"
        step="2"
        bind:value={prefs.size}
        onchange={savePrefs}
        style="--p: {((prefs.size - 12) / 148) * 100}%"
        aria-label={t("size.label")}
      />
      <span class="tabular">{prefs.size}px</span>
      <button
        class="icon"
        aria-label={t("size.reset")}
        onclick={() => {
          prefs.size = 40;
          prefs.previewText = "";
          savePrefs();
        }}><Reset size={14} /></button
      >
    </div>

    <div class="segmented sort" role="group" aria-label={t("sort.label")}>
      {#each sorts as s (s.id)}
        <button
          class:on={ui.sort === s.id}
          onclick={() => {
            ui.sort = s.id;
            refreshRows();
          }}>{s.label}</button
        >
      {/each}
    </div>
  </div>
</div>

<style>
  .header {
    position: relative;
    padding: var(--sp-24) var(--sp-32) var(--sp-16);
    display: flex;
    flex-direction: column;
    gap: var(--sp-16);
    border-bottom: 1px solid var(--line);
  }
  .top {
    display: flex;
    align-items: center;
    gap: var(--sp-24);
  }
  .title {
    flex: 1;
    min-width: 0;
    display: flex;
    align-items: baseline;
    gap: var(--sp-12);
  }
  h1 {
    margin: 0;
    font-family: var(--font-display);
    font-size: 32px;
    line-height: 40px;
    font-weight: 600;
    letter-spacing: -0.02em;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }
  .sub {
    color: var(--text-3);
    font-size: 13px;
    white-space: nowrap;
  }
  .search {
    width: 320px;
    height: 40px;
    display: flex;
    align-items: center;
    gap: var(--sp-8);
    padding: 0 var(--sp-8) 0 var(--sp-16);
    border-radius: var(--r-12);
    background: var(--field);
    box-shadow: inset 0 0 0 1px var(--line-2);
    backdrop-filter: blur(16px);
    color: var(--text-3);
    transition: box-shadow 160ms var(--ease);
  }
  .search:focus-within {
    box-shadow:
      inset 0 0 0 1px var(--accent-line),
      0 0 0 4px var(--accent-ring);
    color: var(--text-2);
  }
  .search input {
    flex: 1;
    min-width: 0;
    background: none;
    border: 0;
    outline: none;
  }
  .search input::placeholder,
  .sample input::placeholder {
    color: var(--text-3);
  }
  kbd {
    display: grid;
    place-items: center;
    width: 24px;
    height: 24px;
    border-radius: var(--r-8);
    background: var(--raised-2);
    font: 500 12px/1 var(--font-ui);
    color: var(--text-3);
  }
  .clear {
    width: 24px;
    height: 24px;
    display: grid;
    place-items: center;
    border-radius: var(--r-8);
  }
  .clear:hover {
    background: var(--press);
    color: var(--text);
  }

  .tools {
    display: flex;
    align-items: center;
    gap: var(--sp-16);
  }
  .segmented {
    display: flex;
    padding: var(--sp-4);
    gap: var(--sp-4);
    border-radius: var(--r-12);
    background: var(--raised);
    box-shadow: inset 0 0 0 1px var(--line);
  }
  .segmented button {
    min-width: 32px;
    height: 32px;
    padding: 0 var(--sp-8);
    display: grid;
    place-items: center;
    border-radius: var(--r-8);
    color: var(--text-3);
    font-size: 13px;
    transition: all 140ms var(--ease);
  }
  .segmented button:hover {
    color: var(--text);
  }
  .segmented button.on {
    background: var(--raised-2);
    color: var(--text);
    box-shadow:
      inset 0 0 0 1px var(--line-2),
      var(--shadow-sm);
  }
  .sample {
    flex: 1;
    min-width: 160px;
    height: 40px;
    display: flex;
    align-items: center;
    gap: var(--sp-8);
    padding: 0 var(--sp-16);
    border-radius: var(--r-12);
    color: var(--text-3);
    transition: background 160ms var(--ease);
  }
  .sample:hover,
  .sample:focus-within {
    background: var(--raised);
  }
  .sample input {
    flex: 1;
    min-width: 0;
    background: none;
    border: 0;
    outline: none;
    color: var(--text);
  }
  .size {
    display: flex;
    align-items: center;
    gap: var(--sp-8);
    color: var(--text-2);
    font-size: 12px;
  }
  .size span {
    width: 40px;
    text-align: right;
  }
  .icon {
    width: 24px;
    height: 24px;
    display: grid;
    place-items: center;
    border-radius: var(--r-8);
    color: var(--text-3);
  }
  .icon:hover {
    color: var(--text);
    background: var(--press);
  }
  input[type="range"] {
    -webkit-appearance: none;
    appearance: none;
    width: 160px;
    height: 24px;
    background: transparent;
  }
  input[type="range"]::-webkit-slider-runnable-track {
    height: 4px;
    border-radius: 4px;
    background: linear-gradient(to right, var(--mint) var(--p), var(--raised-2) var(--p));
  }
  input[type="range"]::-webkit-slider-thumb {
    -webkit-appearance: none;
    width: 16px;
    height: 16px;
    margin-top: -6px;
    border-radius: 50%;
    background: var(--paper);
    box-shadow:
      0 0 0 4px var(--accent-ring),
      var(--shadow-thumb);
  }
  @media (max-width: 1180px) {
    .sort {
      display: none;
    }
  }
</style>
