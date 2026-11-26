<script lang="ts">
  import Logo from "./Logo.svelte";
  import { ui } from "$lib/store.svelte";
  import { t } from "$lib/i18n.svelte";

  const copy = $derived.by(() => {
    if (ui.search) return { t: t("empty.search.t"), d: t("empty.search.d", { q: ui.search }) };
    const keys = { all: "all", favorites: "favorites", active: "active", collection: "collection", google: "google" } as const;
    const key = keys[ui.scope.kind as keyof typeof keys] ?? "default";
    return { t: t(`empty.${key}.t`), d: t(`empty.${key}.d`) };
  });
</script>

<div class="empty">
  <div class="mark"><Logo size={64} ring /></div>
  <h3>{copy.t}</h3>
  <p>{copy.d}</p>
</div>

<style>
  .empty {
    height: 100%;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: var(--sp-8);
    padding: var(--sp-48);
    text-align: center;
  }
  .mark {
    margin-bottom: var(--sp-16);
    border-radius: 15px;
    box-shadow: 0 24px 64px -16px var(--mint-glow);
  }
  h3 {
    margin: 0;
    font-family: var(--font-display);
    font-size: 20px;
    line-height: 32px;
    font-weight: 600;
  }
  p {
    margin: 0;
    max-width: 360px;
    text-wrap: balance;
    color: var(--text-3);
  }
</style>
