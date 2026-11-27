<script lang="ts">
  // Custom title bar for Windows (our own buttons) and macOS (room for the
  // native traffic lights, which overlay the left side). The whole strip
  // drags the window; double-click maximizes.
  import { getCurrentWindow } from "@tauri-apps/api/window";
  import { onMount } from "svelte";
  import { isWindows } from "$lib/platform";
  import { t } from "$lib/i18n.svelte";

  const win = getCurrentWindow();
  let maximized = $state(false);

  onMount(() => {
    win.isMaximized().then((m) => (maximized = m));
    const un = win.onResized(async () => (maximized = await win.isMaximized()));
    return () => {
      un.then((f) => f());
    };
  });
</script>

<div class="titlebar" class:mac={!isWindows} data-tauri-drag-region>
  {#if isWindows}
    <div class="controls">
      <button class="ctl" aria-label={t("win.minimize")} title={t("win.minimize")} onclick={() => win.minimize()}>
        <svg width="10" height="10" viewBox="0 0 10 10"><path d="M1 5.5h8" stroke="currentColor" stroke-width="1" /></svg>
      </button>
      <button
        class="ctl"
        aria-label={maximized ? t("win.restore") : t("win.maximize")}
        title={maximized ? t("win.restore") : t("win.maximize")}
        onclick={() => win.toggleMaximize()}
      >
        {#if maximized}
          <svg width="10" height="10" viewBox="0 0 10 10" fill="none" stroke="currentColor" stroke-width="1">
            <rect x="1.5" y="3.5" width="5" height="5" rx="1" />
            <path d="M3.5 3.5V2.5a1 1 0 0 1 1-1h3a1 1 0 0 1 1 1v3a1 1 0 0 1-1 1h-1" />
          </svg>
        {:else}
          <svg width="10" height="10" viewBox="0 0 10 10" fill="none" stroke="currentColor" stroke-width="1">
            <rect x="1.5" y="1.5" width="7" height="7" rx="1.5" />
          </svg>
        {/if}
      </button>
      <button class="ctl close" aria-label={t("win.close")} title={t("win.close")} onclick={() => win.close()}>
        <svg width="10" height="10" viewBox="0 0 10 10"><path d="M1.5 1.5l7 7M8.5 1.5l-7 7" stroke="currentColor" stroke-width="1" /></svg>
      </button>
    </div>
  {/if}
</div>

<style>
  .titlebar {
    flex: none;
    height: 32px;
    display: flex;
    align-items: center;
    justify-content: flex-end;
    padding: 0 var(--sp-8);
    user-select: none;
  }
  /* macOS: traffic lights sit on the left; give the strip more height for them. */
  .titlebar.mac {
    height: 40px;
  }
  .controls {
    display: flex;
    gap: var(--sp-4);
    padding-top: var(--sp-4);
  }
  .ctl {
    width: 40px;
    height: 24px;
    display: grid;
    place-items: center;
    border-radius: var(--r-8);
    color: var(--text-3);
    transition:
      background 120ms var(--ease),
      color 120ms var(--ease);
  }
  .ctl:hover {
    background: var(--press);
    color: var(--text);
  }
  .ctl.close:hover {
    background: #e5484d;
    color: #fff;
  }
</style>
