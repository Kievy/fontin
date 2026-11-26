<script lang="ts">
  import Download from "@lucide/svelte/icons/download";
  import X from "@lucide/svelte/icons/x";
  import { updater, installUpdate } from "$lib/updater.svelte";
  import { t } from "$lib/i18n.svelte";
</script>

{#if updater.available && !updater.dismissed}
  <div class="banner" role="status">
    <div class="icon"><Download size={16} strokeWidth={2} /></div>
    <div class="text">
      <span class="title">{t("upd.bannerTitle", { v: updater.available.version })}</span>
      <span class="sub">
        {updater.progress === null ? t("upd.bannerSub") : t("upd.downloading", { p: Math.round(updater.progress * 100) })}
      </span>
      {#if updater.progress !== null}
        <span class="bar"><span style="width: {updater.progress * 100}%"></span></span>
      {/if}
    </div>
    {#if updater.progress === null}
      <button class="cta" onclick={installUpdate}>{t("upd.update")}</button>
      <button class="close" aria-label={t("upd.later")} onclick={() => (updater.dismissed = true)}><X size={14} /></button>
    {/if}
  </div>
{/if}

<style>
  .banner {
    position: fixed;
    right: var(--sp-24);
    bottom: var(--sp-24);
    z-index: 90;
    width: 360px;
    display: flex;
    align-items: center;
    gap: var(--sp-12);
    padding: var(--sp-12) var(--sp-12) var(--sp-12) var(--sp-16);
    border-radius: var(--r-16);
    background: var(--toast-bg);
    backdrop-filter: blur(16px);
    box-shadow: var(--shadow-pop);
    animation: up 260ms var(--ease);
  }
  .icon {
    width: 32px;
    height: 32px;
    flex: none;
    display: grid;
    place-items: center;
    border-radius: var(--r-8);
    background: var(--mint);
    color: var(--ink);
  }
  .text {
    flex: 1;
    min-width: 0;
    display: flex;
    flex-direction: column;
  }
  .title {
    font-weight: 600;
    font-size: 13px;
    line-height: 16px;
  }
  .sub {
    font-size: 12px;
    line-height: 16px;
    color: var(--text-3);
  }
  .bar {
    margin-top: var(--sp-4);
    height: 4px;
    border-radius: 4px;
    background: var(--raised-2);
    overflow: hidden;
  }
  .bar span {
    display: block;
    height: 100%;
    background: var(--mint);
    transition: width 160ms var(--ease);
  }
  .cta {
    height: 32px;
    padding: 0 var(--sp-12);
    border-radius: var(--r-8);
    background: var(--mint);
    color: var(--ink);
    font-size: 13px;
    font-weight: 600;
  }
  .close {
    width: 24px;
    height: 24px;
    display: grid;
    place-items: center;
    border-radius: var(--r-8);
    color: var(--text-3);
  }
  .close:hover {
    background: var(--press);
    color: var(--text);
  }
  @keyframes up {
    from {
      opacity: 0;
      transform: translateY(8px);
    }
  }
</style>
