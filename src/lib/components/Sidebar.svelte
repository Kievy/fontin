<script lang="ts">
  import NavItem from "./NavItem.svelte";
  import FolderTree from "./FolderTree.svelte";
  import Layers from "@lucide/svelte/icons/layers";
  import Clock from "@lucide/svelte/icons/clock-3";
  import Heart from "@lucide/svelte/icons/heart";
  import Power from "@lucide/svelte/icons/power";
  import PowerOff from "@lucide/svelte/icons/power-off";
  import Plus from "@lucide/svelte/icons/plus";
  import FolderIcon from "@lucide/svelte/icons/folder";
  import FileType from "@lucide/svelte/icons/file-type";
  import AddMenu from "./AddMenu.svelte";
  import Toggle from "./Toggle.svelte";
  import { t } from "$lib/i18n.svelte";
  import Refresh from "@lucide/svelte/icons/refresh-cw";
  import Trash from "@lucide/svelte/icons/trash-2";
  import Download from "@lucide/svelte/icons/hard-drive-download";
  import Sparkles from "@lucide/svelte/icons/sparkles";
  import { open as openDialog } from "@tauri-apps/plugin-dialog";
  import { SvelteSet } from "svelte/reactivity";
  import { api, type Scope } from "$lib/api";
  import {
    data,
    ui,
    drag,
    prefs,
    savePrefs,
    setScope,
    isScope,
    toast,
    refreshAll,
    addToCollection,
    addToFavorites,
    offerDuplicates,
    locationBusy,
    locationLevel,
    toggleLocation,
  } from "$lib/store.svelte";

  // Resizable width, snapped to the 8px grid.
  const MIN_W = 224;
  const MAX_W = 480;
  const DEFAULT_W = 264;
  const clampW = (w: number) => Math.min(MAX_W, Math.max(MIN_W, Math.round(w / 8) * 8));
  let resizing = $state(false);

  function startResize(e: PointerEvent) {
    if (e.button !== 0) return;
    e.preventDefault();
    const el = e.currentTarget as HTMLElement;
    el.setPointerCapture(e.pointerId);
    const startX = e.clientX;
    const startW = prefs.sidebarWidth;
    resizing = true;
    const move = (ev: PointerEvent) => (prefs.sidebarWidth = clampW(startW + ev.clientX - startX));
    const up = () => {
      resizing = false;
      el.removeEventListener("pointermove", move);
      el.removeEventListener("pointerup", up);
      el.removeEventListener("pointercancel", up);
      savePrefs();
    };
    el.addEventListener("pointermove", move);
    el.addEventListener("pointerup", up);
    el.addEventListener("pointercancel", up);
  }

  function resizeKey(e: KeyboardEvent) {
    const step = e.key === "ArrowLeft" ? -8 : e.key === "ArrowRight" ? 8 : 0;
    if (!step) return;
    e.preventDefault();
    prefs.sidebarWidth = clampW(prefs.sidebarWidth + step);
    savePrefs();
  }

  const COLORS = ["#7df2a6", "#7cc4ff", "#b99cff", "#ffb86b", "#ff7aa8", "#f4f4f0"];
  const CATEGORIES = $derived([
    { id: null, label: t("cat.all") },
    { id: "Sans Serif", label: t("cat.sans") },
    { id: "Serif", label: t("cat.serif") },
    { id: "Display", label: t("cat.display") },
    { id: "Handwriting", label: t("cat.handwriting") },
    { id: "Monospace", label: t("cat.mono") },
  ]);

  const counts = $derived(data.overview?.counts);
  const openFolders = new SvelteSet<string>();

  let creating = $state(false);
  let newName = $state("");

  const libraryNav = $derived([
    { scope: { kind: "all" } as Scope, label: t("scope.all"), icon: Layers, count: counts?.all },
    { scope: { kind: "recent" } as Scope, label: t("scope.recent"), icon: Clock, count: counts?.recent },
    { scope: { kind: "favorites" } as Scope, label: t("scope.favorites"), icon: Heart, count: counts?.favorites },
    { scope: { kind: "active" } as Scope, label: t("scope.active"), icon: Power, count: counts?.active },
    { scope: { kind: "inactive" } as Scope, label: t("scope.inactive"), icon: PowerOff, count: counts?.inactive },
  ]);

  async function addFolder() {
    const path = await openDialog({ directory: true, multiple: false, title: t("add.folderDialog") });
    if (typeof path !== "string") return;
    try {
      const r = await api.addFolder(path);
      setScope({ kind: "source", value: r.sourceId });
      toast(t("toast.folderAdded"), "success");
      offerDuplicates(r.duplicates);
    } catch (e) {
      toast(String(e), "error");
    }
  }

  async function addFonts() {
    const picked = await openDialog({
      multiple: true,
      title: t("add.fontDialog"),
      filters: [{ name: t("add.fontFilter"), extensions: ["ttf", "otf", "ttc", "otc", "TTF", "OTF", "TTC"] }],
    });
    const paths = Array.isArray(picked) ? picked : typeof picked === "string" ? [picked] : [];
    if (!paths.length) return;
    try {
      const r = await api.addFiles(paths);
      // Only jump/announce when something new actually got in.
      if (r.duplicates.length < paths.length) {
        setScope({ kind: "source", value: r.sourceId });
        toast(t("toast.fontsAdded"), "success");
      }
      offerDuplicates(r.duplicates);
    } catch (e) {
      toast(String(e), "error");
    }
  }

  async function rescan(id: number) {
    try {
      offerDuplicates(await api.rescanSource(id));
    } catch (e) {
      toast(String(e), "error");
    }
  }

  function startCollection() {
    if (ui.section !== "library") setScope({ kind: "all" });
    creating = true;
  }

  async function createCollection() {
    const name = newName.trim();
    creating = false;
    newName = "";
    if (!name) return;
    const color = COLORS[(data.overview?.collections.length ?? 0) % COLORS.length];
    try {
      const id = await api.createCollection(name, color);
      setScope({ kind: "collection", value: id });
    } catch (e) {
      toast(String(e), "error");
    }
  }

  async function removeSource(id: number, name: string) {
    if (!confirm(t("side.confirmRemove", { name }))) return;
    try {
      await api.removeSource(id);
      if (isScope(ui.scope, { kind: "source", value: id })) setScope({ kind: "all" });
    } catch (e) {
      toast(String(e), "error");
    }
  }

  async function refreshGoogle() {
    try {
      const n = await api.refreshGoogle(true);
      toast(t("toast.catalogCount", { n }), "success");
      refreshAll();
    } catch (e) {
      toast(t("toast.catalogFail", { e: String(e) }), "error");
    }
  }
</script>

<div class="side-wrap" class:resizing style="--w: {clampW(prefs.sidebarWidth)}px">
<aside class="sidebar">
  {#if ui.section === "library"}
    <header>
      <h2>{t("nav.library")}</h2>
      <p>{t("lib.activeNow", { n: counts?.active ?? 0 })}</p>
    </header>

    <div class="scroll">
      <section>
        {#each libraryNav as item (item.label)}
          <NavItem
            active={isScope(ui.scope, item.scope)}
            count={item.count ?? null}
            onclick={() => setScope(item.scope)}
            ondropfamily={item.scope.kind === "favorites" ? addToFavorites : undefined}
          >
            {#snippet icon()}<item.icon size={16} strokeWidth={1.75} />{/snippet}
            {item.label}
          </NavItem>
        {/each}
      </section>

      <section>
        <div class="section-head">
          <span class="eyebrow">{t("side.collections")}</span>
          <button class="mini" aria-label={t("side.newCollection")} onclick={() => (creating = true)}><Plus size={14} /></button>
        </div>
        {#each data.overview?.collections ?? [] as c (c.id)}
          <NavItem
            active={isScope(ui.scope, { kind: "collection", value: c.id })}
            count={c.count}
            onclick={() => setScope({ kind: "collection", value: c.id })}
            ondropfamily={(row) => addToCollection(c.id, row)}
          >
            {#snippet icon()}<span class="swatch" style="--c: {c.color}" class:lit={c.active > 0}></span>{/snippet}
            {#snippet actions()}
              <button
                class="mini"
                aria-label={t("side.deleteCollection")}
                onclick={async () => {
                  if (!confirm(t("side.confirmDeleteCollection", { name: c.name }))) return;
                  await api.deleteCollection(c.id);
                  if (isScope(ui.scope, { kind: "collection", value: c.id })) setScope({ kind: "all" });
                }}><Trash size={14} /></button
              >
            {/snippet}
            {c.name}
          </NavItem>
        {/each}
        {#if drag.row && data.overview?.collections.length}
          <p class="hint">{t("side.dropHint")}</p>
        {/if}
        {#if creating}
          <!-- svelte-ignore a11y_autofocus -->
          <input
            class="new"
            placeholder={t("side.collectionName")}
            autofocus
            bind:value={newName}
            onblur={createCollection}
            onkeydown={(e) => {
              if (e.key === "Enter") createCollection();
              if (e.key === "Escape") {
                newName = "";
                creating = false;
              }
            }}
          />
        {:else if !data.overview?.collections.length}
          <button class="ghost" onclick={() => (creating = true)}>{t("side.newCollection")}</button>
        {/if}
      </section>

      <section>
        <div class="section-head">
          <span class="eyebrow">{t("side.folders")}</span>
          <button class="mini" aria-label={t("add.folder")} onclick={addFolder}><Plus size={14} /></button>
        </div>
        {#each data.overview?.sources ?? [] as s (s.id)}
          <NavItem
            active={isScope(ui.scope, { kind: "source", value: s.id })}
            count={s.count}
            onclick={() => setScope({ kind: "source", value: s.id })}
          >
            {#snippet icon()}
              <span class:spin={data.scanning[s.id]}>
                {#if data.scanning[s.id]}<Refresh size={14} />{:else if s.kind === "files"}<FileType
                    size={16}
                    strokeWidth={1.75}
                  />{:else}<FolderIcon size={16} strokeWidth={1.75} />{/if}
              </span>
            {/snippet}
            {#snippet actions()}
              <button class="mini" aria-label={t("side.rescan")} onclick={() => rescan(s.id)}><Refresh size={14} /></button>
              <button
                class="mini"
                aria-label={t("side.removeFolder")}
                onclick={() => removeSource(s.id, s.kind === "files" ? t("side.looseFiles") : s.name)}><Trash size={14} /></button
              >
            {/snippet}
            {#snippet trailing()}
              {@const level = locationLevel(s.active, s.count)}
              <Toggle
                size="sm"
                state={level}
                busy={locationBusy.has(`${s.id}:`)}
                label={level === 2 ? t("side.deactivateFolder") : t("side.activateFolder")}
                onchange={() => toggleLocation(s.id, null, level)}
              />
            {/snippet}
            {s.kind === "files" ? t("side.looseFiles") : s.name}
          </NavItem>
          <FolderTree nodes={s.tree} source={s.id} open={openFolders} />
        {/each}
      </section>
    </div>

    <footer>
      <AddMenu onfolder={addFolder} onfont={addFonts} oncollection={startCollection} />
    </footer>
  {:else}
    <header>
      <h2>{t("nav.google")}</h2>
      <p>
        {#if data.overview?.googleReady}
          {t("google.subtitle", { n: counts?.google ?? 0 })}
        {:else}
          {t("google.loading")}
        {/if}
      </p>
    </header>
    <div class="scroll">
      <section>
        <NavItem
          active={isScope(ui.scope, { kind: "google" })}
          count={counts?.google ?? null}
          onclick={() => setScope({ kind: "google" }, "google")}
        >
          {#snippet icon()}<Sparkles size={16} strokeWidth={1.75} />{/snippet}
          {t("google.catalog")}
        </NavItem>
        <NavItem
          active={isScope(ui.scope, { kind: "source", value: 1 })}
          count={counts?.googleActive ?? null}
          onclick={() => setScope({ kind: "source", value: 1 }, "google")}
        >
          {#snippet icon()}<Download size={16} strokeWidth={1.75} />{/snippet}
          {t("google.cached")}
        </NavItem>
      </section>
      <section>
        <div class="section-head"><span class="eyebrow">{t("google.categories")}</span></div>
        {#each CATEGORIES as cat (cat.id)}
          <NavItem
            active={ui.category === cat.id}
            onclick={() => {
              ui.category = cat.id;
              setScope(ui.scope.kind === "source" ? ui.scope : { kind: "google" }, "google");
            }}
          >
            {#snippet icon()}<span class="glyph" class:on={ui.category === cat.id}>Aa</span>{/snippet}
            {cat.label}
          </NavItem>
        {/each}
      </section>
    </div>
    <footer>
      <button class="add" onclick={refreshGoogle}>
        <Refresh size={16} strokeWidth={1.75} />
        {t("google.refresh")}
      </button>
    </footer>
  {/if}
</aside>
<!-- svelte-ignore a11y_no_noninteractive_tabindex, a11y_no_noninteractive_element_interactions -->
<div
  class="resizer"
  role="separator"
  aria-orientation="vertical"
  aria-valuenow={prefs.sidebarWidth}
  aria-valuemin={MIN_W}
  aria-valuemax={MAX_W}
  aria-label={t("side.resize")}
  title={t("side.resize")}
  tabindex="0"
  onpointerdown={startResize}
  ondblclick={() => {
    prefs.sidebarWidth = DEFAULT_W;
    savePrefs();
  }}
  onkeydown={resizeKey}
></div>
</div>

<style>
  .side-wrap {
    position: relative;
    flex: none;
    display: flex;
    width: var(--w);
  }
  .sidebar {
    flex: 1;
    min-width: 0;
    display: flex;
    flex-direction: column;
    background: var(--panel);
    border: 1px solid var(--line);
    border-radius: var(--r-16);
    overflow: hidden;
  }
  header {
    padding: var(--sp-24) var(--sp-24) var(--sp-16);
  }
  h2 {
    margin: 0;
    font-family: var(--font-display);
    font-size: 20px;
    line-height: 32px;
    font-weight: 600;
    letter-spacing: -0.01em;
  }
  header p {
    margin: 0;
    font-size: 12px;
    line-height: 16px;
    color: var(--text-3);
  }
  .scroll {
    flex: 1;
    overflow-y: auto;
    padding: 0 var(--sp-8) var(--sp-16);
  }
  section {
    display: flex;
    flex-direction: column;
    gap: 2px;
    padding: var(--sp-8) var(--sp-8);
  }
  section + section {
    margin-top: var(--sp-8);
  }
  .section-head {
    display: flex;
    align-items: center;
    justify-content: space-between;
    height: 32px;
    padding: 0 var(--sp-4) 0 var(--sp-12);
  }
  .mini {
    width: 24px;
    height: 24px;
    display: grid;
    place-items: center;
    border-radius: var(--r-8);
    color: var(--text-3);
  }
  .mini:hover {
    background: var(--press);
    color: var(--text);
  }
  .swatch {
    width: 8px;
    height: 8px;
    border-radius: 50%;
    box-shadow: inset 0 0 0 1.5px var(--c);
  }
  .swatch.lit {
    background: var(--c);
    box-shadow: 0 0 8px var(--c);
  }
  .glyph {
    font-family: var(--font-display);
    font-size: 12px;
    font-weight: 700;
    color: var(--text-3);
  }
  .glyph.on {
    color: var(--accent-text);
  }
  .new {
    height: 32px;
    margin: 0 0 0 var(--sp-4);
    padding: 0 var(--sp-12);
    border-radius: var(--r-8);
    border: 1px solid var(--accent-text);
    background: var(--raised);
    outline: none;
  }
  .hint {
    margin: var(--sp-4) 0 0;
    padding: 0 var(--sp-12);
    font-size: 12px;
    line-height: 16px;
    color: var(--accent-text);
  }
  .ghost {
    height: 32px;
    padding: 0 var(--sp-12);
    text-align: left;
    color: var(--text-3);
    border-radius: var(--r-8);
    border: 1px dashed var(--line-2);
  }
  .ghost:hover {
    color: var(--text);
  }
  footer {
    padding: var(--sp-16);
    border-top: 1px solid var(--line);
  }
  .add {
    width: 100%;
    height: 40px;
    display: flex;
    align-items: center;
    justify-content: center;
    gap: var(--sp-8);
    border-radius: var(--r-12);
    background: var(--raised);
    box-shadow: inset 0 0 0 1px var(--line-2);
    color: var(--text-2);
    font-weight: 500;
    transition: all 160ms var(--ease);
  }
  .add:hover {
    color: var(--text);
    background: var(--raised-2);
  }
  /* Sits in the 8px gap between the sidebar and the main panel. */
  .resizer {
    position: absolute;
    top: 0;
    bottom: 0;
    right: calc(var(--sp-8) * -1);
    width: var(--sp-8);
    cursor: col-resize;
    z-index: 2;
    outline: none;
    touch-action: none;
  }
  .resizer::after {
    content: "";
    position: absolute;
    top: var(--sp-16);
    bottom: var(--sp-16);
    left: 50%;
    width: 2px;
    margin-left: -1px;
    border-radius: 2px;
    background: var(--mint);
    opacity: 0;
    transition: opacity 140ms var(--ease);
  }
  .resizer:hover::after,
  .resizer:focus-visible::after,
  .resizing .resizer::after {
    opacity: 0.8;
    box-shadow: 0 0 8px var(--mint-glow);
  }
  .resizing {
    user-select: none;
  }
  :global(body:has(.side-wrap.resizing)) {
    cursor: col-resize;
  }
  .spin {
    display: grid;
    animation: spin 1s linear infinite;
    color: var(--accent-text);
  }
  @keyframes spin {
    to {
      rotate: 360deg;
    }
  }
</style>
