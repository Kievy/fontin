<script lang="ts">
  import { onMount } from "svelte";
  import Rail from "$lib/components/Rail.svelte";
  import Sidebar from "$lib/components/Sidebar.svelte";
  import Header from "$lib/components/Header.svelte";
  import FontList from "$lib/components/FontList.svelte";
  import DetailPanel from "$lib/components/DetailPanel.svelte";
  import Toasts from "$lib/components/Toasts.svelte";
  import Settings from "$lib/components/Settings.svelte";
  import UpdateBanner from "$lib/components/UpdateBanner.svelte";
  import TitleBar from "$lib/components/TitleBar.svelte";
  import { customTitleBar } from "$lib/platform";
  import { startUpdateChecks } from "$lib/updater.svelte";
  import { init, ui, data, select, setScope } from "$lib/store.svelte";

  let searchEl: HTMLInputElement | undefined = $state();

  onMount(() => {
    init();
    startUpdateChecks();
  });

  function onKey(e: KeyboardEvent) {
    const typing = (e.target as HTMLElement)?.tagName === "INPUT";
    if (!typing && (e.key === "/" || ((e.ctrlKey || e.metaKey) && e.key === "k"))) {
      e.preventDefault();
      searchEl?.focus();
    } else if ((e.ctrlKey || e.metaKey) && (e.key === "1" || e.key === "2" || e.key === ",")) {
      // Ctrl/⌘+1 Library · Ctrl/⌘+2 Google Fonts · Ctrl/⌘+, Settings
      e.preventDefault();
      ui.settingsOpen = e.key === ",";
      if (e.key === "1") setScope({ kind: "all" });
      if (e.key === "2") setScope({ kind: "google" }, "google");
    } else if (!typing && e.key === "Escape" && ui.selected) {
      select(null);
    }
  }
</script>

<svelte:window onkeydown={onKey} />

<div class="shell">
{#if customTitleBar}<TitleBar />{/if}
<div class="app" class:framed={customTitleBar}>
  <Rail />
  {#if !ui.settingsOpen}<Sidebar />{/if}
  <main>
    <div class="aura"></div>
    {#if ui.settingsOpen}
      <Settings />
    {:else}
      <Header bind:searchEl />
      <FontList />
    {/if}
  </main>
  {#if data.detail && !ui.settingsOpen}<DetailPanel />{/if}
</div>
</div>
<Toasts />
<UpdateBanner />

<style>
  .shell {
    height: 100vh;
    display: flex;
    flex-direction: column;
    background:
      radial-gradient(1200px 600px at 100% -10%, var(--aura-bg), transparent 60%),
      var(--bg);
  }
  .app {
    flex: 1;
    min-height: 0;
    display: flex;
    gap: var(--sp-8);
    padding: var(--sp-8);
  }
  /* The title bar above already provides the top spacing. */
  .app.framed {
    padding-top: 0;
  }
  main {
    position: relative;
    flex: 1;
    min-width: 0;
    display: flex;
    flex-direction: column;
    background: var(--panel);
    border: 1px solid var(--line);
    border-radius: var(--r-16);
    overflow: hidden;
    isolation: isolate;
  }
  .aura {
    position: absolute;
    inset: 0 0 auto 0;
    height: 240px;
    background:
      radial-gradient(600px 240px at 85% -40%, var(--aura-1), transparent 70%),
      radial-gradient(400px 200px at 10% -60%, var(--aura-2), transparent 70%);
    pointer-events: none;
    z-index: -1;
  }
</style>
