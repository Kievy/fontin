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
  import FolderPlus from "@lucide/svelte/icons/folder-plus";
  import Refresh from "@lucide/svelte/icons/refresh-cw";
  import Trash from "@lucide/svelte/icons/trash-2";
  import Download from "@lucide/svelte/icons/hard-drive-download";
  import Sparkles from "@lucide/svelte/icons/sparkles";
  import { open as openDialog } from "@tauri-apps/plugin-dialog";
  import { SvelteSet } from "svelte/reactivity";
  import { api, type Scope } from "$lib/api";
  import { data, ui, drag, setScope, isScope, toast, refreshAll, addToCollection, addToFavorites } from "$lib/store.svelte";

  const COLORS = ["#7df2a6", "#7cc4ff", "#b99cff", "#ffb86b", "#ff7aa8", "#f4f4f0"];
  const CATEGORIES = [
    { id: null, label: "Todas as categorias" },
    { id: "Sans Serif", label: "Sans-serif" },
    { id: "Serif", label: "Serif" },
    { id: "Display", label: "Display" },
    { id: "Handwriting", label: "Manuscrita" },
    { id: "Monospace", label: "Monoespaçada" },
  ];

  const counts = $derived(data.overview?.counts);
  const openFolders = new SvelteSet<string>();

  let creating = $state(false);
  let newName = $state("");

  const libraryNav = $derived([
    { scope: { kind: "all" } as Scope, label: "Todas", icon: Layers, count: counts?.all },
    { scope: { kind: "recent" } as Scope, label: "Recentes", icon: Clock, count: counts?.recent },
    { scope: { kind: "favorites" } as Scope, label: "Favoritas", icon: Heart, count: counts?.favorites },
    { scope: { kind: "active" } as Scope, label: "Ativas", icon: Power, count: counts?.active },
    { scope: { kind: "inactive" } as Scope, label: "Inativas", icon: PowerOff, count: counts?.inactive },
  ]);

  async function addFolder() {
    const path = await openDialog({ directory: true, multiple: false, title: "Adicionar pasta de fontes" });
    if (typeof path !== "string") return;
    try {
      const id = await api.addFolder(path);
      setScope({ kind: "source", value: id });
      toast("Pasta adicionada", "success");
    } catch (e) {
      toast(String(e), "error");
    }
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
    if (!confirm(`Remover “${name}” da biblioteca? Os arquivos não são apagados.`)) return;
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
      toast(`${n.toLocaleString("pt-BR")} famílias no catálogo`, "success");
      refreshAll();
    } catch (e) {
      toast(`Catálogo indisponível: ${e}`, "error");
    }
  }
</script>

<aside class="sidebar">
  {#if ui.section === "library"}
    <header>
      <h2>Biblioteca</h2>
      <p>{(counts?.active ?? 0).toLocaleString("pt-BR")} ativas agora</p>
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
          <span class="eyebrow">Coleções</span>
          <button class="mini" aria-label="Nova coleção" onclick={() => (creating = true)}><Plus size={14} /></button>
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
                aria-label="Excluir coleção"
                onclick={async () => {
                  if (!confirm(`Excluir a coleção “${c.name}”?`)) return;
                  await api.deleteCollection(c.id);
                  if (isScope(ui.scope, { kind: "collection", value: c.id })) setScope({ kind: "all" });
                }}><Trash size={14} /></button
              >
            {/snippet}
            {c.name}
          </NavItem>
        {/each}
        {#if drag.row && data.overview?.collections.length}
          <p class="hint">Solte em uma coleção</p>
        {/if}
        {#if creating}
          <!-- svelte-ignore a11y_autofocus -->
          <input
            class="new"
            placeholder="Nome da coleção"
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
          <button class="ghost" onclick={() => (creating = true)}>Nova coleção</button>
        {/if}
      </section>

      <section>
        <div class="section-head">
          <span class="eyebrow">Pastas</span>
          <button class="mini" aria-label="Adicionar pasta" onclick={addFolder}><Plus size={14} /></button>
        </div>
        {#each data.overview?.sources ?? [] as s (s.id)}
          <NavItem
            active={isScope(ui.scope, { kind: "source", value: s.id })}
            count={s.count}
            onclick={() => setScope({ kind: "source", value: s.id })}
          >
            {#snippet icon()}
              <span class:spin={data.scanning[s.id]}>
                {#if data.scanning[s.id]}<Refresh size={14} />{:else}<FolderIcon size={16} strokeWidth={1.75} />{/if}
              </span>
            {/snippet}
            {#snippet actions()}
              <button class="mini" aria-label="Reescanear" onclick={() => api.rescanSource(s.id)}><Refresh size={14} /></button>
              <button class="mini" aria-label="Remover pasta" onclick={() => removeSource(s.id, s.name)}><Trash size={14} /></button>
            {/snippet}
            {s.name}
          </NavItem>
          <FolderTree nodes={s.tree} open={openFolders} />
        {/each}
      </section>
    </div>

    <footer>
      <button class="add" onclick={addFolder}>
        <FolderPlus size={16} strokeWidth={1.75} />
        Adicionar pasta
      </button>
    </footer>
  {:else}
    <header>
      <h2>Google Fonts</h2>
      <p>
        {#if data.overview?.googleReady}
          {(counts?.google ?? 0).toLocaleString("pt-BR")} famílias · ative sem baixar
        {:else}
          Carregando catálogo…
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
          Catálogo
        </NavItem>
        <NavItem
          active={isScope(ui.scope, { kind: "source", value: 1 })}
          count={counts?.googleActive ?? null}
          onclick={() => setScope({ kind: "source", value: 1 }, "google")}
        >
          {#snippet icon()}<Download size={16} strokeWidth={1.75} />{/snippet}
          Em cache
        </NavItem>
      </section>
      <section>
        <div class="section-head"><span class="eyebrow">Categorias</span></div>
        {#each CATEGORIES as cat (cat.label)}
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
        Atualizar catálogo
      </button>
    </footer>
  {/if}
</aside>

<style>
  .sidebar {
    width: 264px;
    flex: none;
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
