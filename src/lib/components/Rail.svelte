<script lang="ts">
  import Logo from "./Logo.svelte";
  import LibraryIcon from "@lucide/svelte/icons/library-big";
  import GoogleIcon from "./GoogleIcon.svelte";
  import Settings from "@lucide/svelte/icons/settings-2";
  import { ui, setScope } from "$lib/store.svelte";
  import { t } from "$lib/i18n.svelte";

  const items = $derived([
    { id: "library", label: t("nav.library"), go: () => setScope({ kind: "all" }) },
    { id: "google", label: t("nav.google"), go: () => setScope({ kind: "google" }, "google") },
  ] as const);
</script>

<nav class="rail">
  <div class="logo"><Logo size={40} ring /></div>
  <div class="items">
    {#each items as item (item.id)}
      <button
        class="item"
        class:active={ui.section === item.id && !ui.settingsOpen}
        aria-label={item.label}
        onclick={() => {
          ui.settingsOpen = false;
          item.go();
        }}
      >
        {#if item.id === "google"}
          <GoogleIcon size={18} mono={ui.section === "google" && !ui.settingsOpen} />
        {:else}
          <LibraryIcon size={20} strokeWidth={1.75} />
        {/if}
        <span class="tip">{item.label}</span>
      </button>
    {/each}
  </div>
  <div class="bottom">
    <button
      class="item"
      class:active={ui.settingsOpen}
      aria-label={t("nav.settings")}
      onclick={() => (ui.settingsOpen = !ui.settingsOpen)}
    >
      <Settings size={20} strokeWidth={1.75} />
      <span class="tip">{t("nav.settings")}</span>
    </button>
  </div>
</nav>

<style>
  .rail {
    width: 72px;
    flex: none;
    display: flex;
    flex-direction: column;
    align-items: center;
    padding: var(--sp-16) 0;
    gap: var(--sp-32);
    background: var(--panel);
    border: 1px solid var(--line);
    border-radius: var(--r-16);
  }
  .logo {
    display: grid;
    place-items: center;
  }
  .items,
  .bottom {
    display: flex;
    flex-direction: column;
    gap: var(--sp-8);
  }
  .bottom {
    margin-top: auto;
  }
  .item {
    position: relative;
    width: 40px;
    height: 40px;
    display: grid;
    place-items: center;
    border-radius: var(--r-12);
    color: var(--text-3);
    box-shadow: inset 0 0 0 1px var(--line);
    transition:
      background 160ms var(--ease),
      color 160ms var(--ease);
  }
  .item:hover {
    color: var(--text);
    background: var(--hover);
  }
  .item.active {
    background: var(--mint);
    color: var(--ink);
    box-shadow: 0 0 24px -6px var(--mint-glow);
  }
  .tip {
    position: absolute;
    left: calc(100% + var(--sp-12));
    top: 50%;
    translate: 0 -50%;
    padding: var(--sp-4) var(--sp-8);
    border-radius: var(--r-8);
    background: var(--raised-2);
    box-shadow: var(--shadow-pop);
    color: var(--text);
    font-size: 12px;
    line-height: 16px;
    white-space: nowrap;
    pointer-events: none;
    opacity: 0;
    transform: translateX(-4px);
    transition:
      opacity 120ms var(--ease),
      transform 120ms var(--ease);
    z-index: 20;
  }
  .item:hover .tip {
    opacity: 1;
    transform: none;
  }
</style>
