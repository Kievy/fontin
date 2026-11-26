<script lang="ts">
  import X from "@lucide/svelte/icons/x";
  import Heart from "@lucide/svelte/icons/heart";
  import FolderOpen from "@lucide/svelte/icons/folder-open";
  import Check from "@lucide/svelte/icons/check";
  import Toggle from "./Toggle.svelte";
  import { revealItemInDir } from "@tauri-apps/plugin-opener";
  import { api, type FaceInfo } from "$lib/api";
  import { loadLocalFace, localFamily, loadRow, rowFamily } from "$lib/fonts";
  import { data, select, toggleFamily, toggleFavorite, toast, refreshDetail, previewText } from "$lib/store.svelte";
  import { t } from "$lib/i18n.svelte";

  const d = $derived(data.detail);
  const row = $derived(d?.row);
  const family = $derived(row ? rowFamily(row) : null);
  let ready = $state(false);

  $effect(() => {
    if (!row) return;
    ready = false;
    loadRow(row).then((ok) => (ready = ok));
  });

  // Load each style's face lazily for the per-style previews.
  let loadedFaces = $state<Record<string, boolean>>({});
  $effect(() => {
    for (const f of d?.faces ?? []) {
      if (!loadedFaces[f.id]) loadLocalFace(f.id).then((ok) => ok && (loadedFaces[f.id] = true));
    }
  });

  async function toggleFace(f: FaceInfo) {
    try {
      await api.setFaceActive(f.id, !f.active);
    } catch (e) {
      toast(String(e), "error");
    }
  }

  async function toggleCollection(id: number, member: boolean) {
    if (!row) return;
    await api.setInCollection(id, [row.id], !member);
    refreshDetail();
  }

  const folder = $derived(d?.faces[0]?.path);
  const fmtSize = (n: number) => (n > 1e6 ? `${(n / 1e6).toFixed(1)} MB` : `${Math.round(n / 1e3)} KB`);
  const totalSize = $derived((d?.faces ?? []).reduce((a, f) => a + f.size, 0));
</script>

{#if d && row}
  <aside class="panel">
    <div class="glow"></div>
    <header>
      <div class="actions">
        <span class="eyebrow">{row.provider === "google" ? `Google · ${row.category ?? ""}` : row.format}</span>
        <button class="icon" aria-label={t("detail.close")} onclick={() => select(null)}><X size={16} /></button>
      </div>
      <h2 class:ready style="font-family: {family ? `'${family}', ` : ''}var(--font-display)">{row.name}</h2>
      <div class="controls">
        <Toggle
          state={row.active}
          busy={!!data.pending[row.id] || !!data.downloads[row.id]}
          label={t("detail.activateFamily")}
          onchange={() => toggleFamily(row)}
        />
        <span class="state">
          {row.active === 2 ? t("detail.activeSystem") : row.active === 1 ? t("detail.partial") : t("detail.inactive")}
        </span>
        <button class="icon fav" class:on={row.favorite} aria-label={row.favorite ? t("fav.remove") : t("fav.add")} onclick={() => toggleFavorite(row)}>
          <Heart size={16} fill={row.favorite ? "currentColor" : "none"} />
        </button>
      </div>
    </header>

    <div class="scroll">
      <div class="specimen" class:ready style="font-family: {family ? `'${family}', ` : ''}var(--font-ui)">
        <div class="aa">Aa</div>
        <div class="glyphs">ABCDEFGHIJKLMNOPQRSTUVWXYZ<br />abcdefghijklmnopqrstuvwxyz<br />0123456789 &amp;@!?€$ áçõü</div>
      </div>

      <section>
        <div class="head">
          <span class="eyebrow">{t("detail.styles")}</span>
          <span class="muted tabular">{d.faces.length || row.styles}</span>
        </div>
        {#if d.faces.length}
          <ul class="faces">
            {#each d.faces as f (f.id)}
              <li>
                <Toggle size="sm" state={f.active} label={t("detail.activateStyle")} onchange={() => toggleFace(f)} />
                <div class="face">
                  <span class="style">{f.style}<em class="tabular">{f.weight}{f.variable ? " · VF" : ""}</em></span>
                  <span
                    class="sample"
                    class:ready={loadedFaces[f.id]}
                    style="font-family: '{localFamily(f.id)}', var(--font-ui)">{previewText()}</span
                  >
                </div>
              </li>
            {/each}
          </ul>
        {:else}
          <p class="note">
            {t("detail.googleNote", { n: row.styles })}
          </p>
        {/if}
      </section>

      {#if data.overview?.collections.length}
        <section>
          <div class="head"><span class="eyebrow">{t("detail.collections")}</span></div>
          <div class="chips">
            {#each data.overview.collections as c (c.id)}
              {@const member = d.collections.includes(c.id)}
              <button class="chip" class:on={member} style="--c: {c.color}" onclick={() => toggleCollection(c.id, member)}>
                {#if member}<Check size={12} />{:else}<span class="dot"></span>{/if}
                {c.name}
              </button>
            {/each}
          </div>
        </section>
      {/if}

      <section>
        <div class="head"><span class="eyebrow">{t("detail.details")}</span></div>
        <dl>
          {#if d.designers.length}<dt>{t("detail.design")}</dt><dd>{d.designers.join(", ")}</dd>{/if}
          {#if d.faces.length}<dt>{t("detail.files")}</dt><dd class="tabular">{d.faces.length} · {fmtSize(totalSize)}</dd>{/if}
          {#if d.faces[0]?.postscript}<dt>PostScript</dt><dd class="mono">{d.faces[0].postscript}</dd>{/if}
        </dl>
        {#if folder}
          <button class="reveal" onclick={() => revealItemInDir(folder)}>
            <FolderOpen size={16} strokeWidth={1.75} />
            {t("detail.reveal")}
          </button>
        {/if}
      </section>
    </div>
  </aside>
{/if}

<style>
  .panel {
    position: relative;
    width: 360px;
    flex: none;
    display: flex;
    flex-direction: column;
    background: var(--panel);
    border: 1px solid var(--line);
    border-radius: var(--r-16);
    overflow: hidden;
    animation: enter 240ms var(--ease);
  }
  @keyframes enter {
    from {
      opacity: 0;
      transform: translateX(16px);
    }
  }
  .glow {
    position: absolute;
    inset: -120px -80px auto auto;
    width: 320px;
    height: 320px;
    background: radial-gradient(circle, var(--aura-1), transparent 65%);
    pointer-events: none;
  }
  header {
    position: relative;
    padding: var(--sp-24) var(--sp-24) var(--sp-16);
    display: flex;
    flex-direction: column;
    gap: var(--sp-8);
  }
  .actions {
    display: flex;
    align-items: center;
    justify-content: space-between;
  }
  h2 {
    margin: 0;
    font-size: 32px;
    line-height: 40px;
    font-weight: 400;
    overflow-wrap: anywhere;
    opacity: 0.4;
    transition: opacity 200ms var(--ease);
  }
  h2.ready {
    opacity: 1;
  }
  .controls {
    display: flex;
    align-items: center;
    gap: var(--sp-12);
    margin-top: var(--sp-8);
  }
  .state {
    flex: 1;
    font-size: 13px;
    color: var(--text-2);
  }
  .icon {
    width: 32px;
    height: 32px;
    display: grid;
    place-items: center;
    border-radius: var(--r-8);
    color: var(--text-3);
  }
  .icon:hover {
    background: var(--press);
    color: var(--text);
  }
  .fav.on {
    color: var(--heart);
  }
  .scroll {
    flex: 1;
    overflow-y: auto;
    padding: 0 var(--sp-24) var(--sp-24);
  }
  .specimen {
    padding: var(--sp-24);
    border-radius: var(--r-16);
    background: linear-gradient(160deg, var(--raised-2), var(--raised));
    box-shadow: inset 0 0 0 1px var(--line);
    opacity: 0.4;
    transition: opacity 200ms var(--ease);
    user-select: text;
  }
  .specimen.ready {
    opacity: 1;
  }
  .aa {
    font-size: 96px;
    line-height: 104px;
    color: var(--text);
  }
  .glyphs {
    margin-top: var(--sp-8);
    font-size: 16px;
    line-height: 24px;
    color: var(--text-2);
    word-break: break-all;
  }
  section {
    margin-top: var(--sp-24);
  }
  .head {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: var(--sp-8);
  }
  .muted {
    color: var(--text-3);
    font-size: 12px;
  }
  .faces {
    list-style: none;
    margin: 0;
    padding: 0;
    display: flex;
    flex-direction: column;
  }
  .faces li {
    display: flex;
    align-items: flex-start;
    gap: var(--sp-12);
    padding: var(--sp-12) 0;
    border-top: 1px solid var(--line);
  }
  .faces li :global(.toggle) {
    margin-top: 4px;
  }
  .face {
    flex: 1;
    min-width: 0;
    display: flex;
    flex-direction: column;
  }
  .style {
    font-size: 12px;
    line-height: 16px;
    color: var(--text-2);
  }
  .style em {
    font-style: normal;
    color: var(--text-4);
    margin-left: var(--sp-8);
  }
  .sample {
    font-size: 20px;
    line-height: 32px;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    opacity: 0;
    transition: opacity 200ms var(--ease);
  }
  .sample.ready {
    opacity: 1;
  }
  .note {
    margin: 0;
    color: var(--text-3);
    font-size: 13px;
  }
  .chips {
    display: flex;
    flex-wrap: wrap;
    gap: var(--sp-8);
  }
  .chip {
    height: 32px;
    display: inline-flex;
    align-items: center;
    gap: var(--sp-8);
    padding: 0 var(--sp-12);
    border-radius: var(--r-full);
    box-shadow: inset 0 0 0 1px var(--line-2);
    color: var(--text-2);
    font-size: 13px;
  }
  .chip:hover {
    color: var(--text);
  }
  .chip.on {
    color: var(--ink);
    background: var(--c);
    box-shadow: none;
  }
  .chip .dot {
    width: 8px;
    height: 8px;
    border-radius: 50%;
    background: var(--c);
  }
  dl {
    display: grid;
    grid-template-columns: 96px 1fr;
    gap: var(--sp-8) var(--sp-16);
    margin: 0;
    font-size: 13px;
  }
  dt {
    color: var(--text-3);
  }
  dd {
    margin: 0;
    color: var(--text-2);
    overflow-wrap: anywhere;
    user-select: text;
  }
  .mono {
    font-family: ui-monospace, monospace;
    font-size: 12px;
  }
  .reveal {
    margin-top: var(--sp-16);
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
  }
  .reveal:hover {
    color: var(--text);
    background: var(--raised-2);
  }
</style>
