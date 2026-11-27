<script lang="ts">
  import Logo from "./Logo.svelte";
  import Toggle from "./Toggle.svelte";
  import { onMount } from "svelte";
  import { getVersion } from "@tauri-apps/api/app";
  import { openUrl } from "@tauri-apps/plugin-opener";
  import { updater, checkForUpdates, installUpdate } from "$lib/updater.svelte";
  import { api, type AppSettings } from "$lib/api";
  import { data, toast, prefs, setTheme, setLanguage } from "$lib/store.svelte";
  import { t, i18n, LOCALES, type LangPref } from "$lib/i18n.svelte";
  import type { ThemePref } from "$lib/theme";
  import Monitor from "@lucide/svelte/icons/monitor";
  import Sun from "@lucide/svelte/icons/sun";
  import Moon from "@lucide/svelte/icons/moon";

  const themes = $derived<{ id: ThemePref; label: string; icon: typeof Sun }[]>([
    { id: "system", label: t("theme.system"), icon: Monitor },
    { id: "light", label: t("theme.light"), icon: Sun },
    { id: "dark", label: t("theme.dark"), icon: Moon },
  ]);
  const languages = $derived<{ id: LangPref; label: string }[]>([
    { id: "system", label: t("lang.system") },
    ...LOCALES,
  ]);

  let settings = $state<AppSettings | null>(null);
  let saving = $state<BoolKey | null>(null);

  let version = $state("");
  onMount(async () => {
    getVersion().then((v) => (version = v));
    try {
      settings = await api.getSettings();
    } catch (e) {
      toast(String(e), "error");
    }
  });

  type BoolKey = "hideOnClose" | "autostart" | "startHidden" | "deactivateOnExit";
  const optionKeys: BoolKey[] = ["hideOnClose", "autostart", "startHidden", "deactivateOnExit"];
  const options = $derived(
    optionKeys.map((key) => ({ key, title: t(`opt.${key}.t`), desc: t(`opt.${key}.d`) })),
  );

  async function flip(key: BoolKey) {
    if (!settings || saving) return;
    const next = { ...settings, [key]: !settings[key] };
    saving = key;
    try {
      await api.setSettings(next);
      settings = next;
    } catch (e) {
      toast(String(e), "error");
    } finally {
      saving = null;
    }
  }
</script>

<div class="settings">
  <div class="hero">
    <Logo size={64} />
    <div>
      <h1>fontin</h1>
      <p>{t("set.tagline")}</p>
    </div>
  </div>

  <section>
    <span class="eyebrow">{t("set.appearance")}</span>
    <div class="group">
      <div class="option">
        <div class="text">
          <span class="title">{t("set.theme")}</span>
          <span class="desc">{t("set.themeDesc")}</span>
        </div>
        <div class="segmented" role="radiogroup" aria-label={t("set.theme")}>
          {#each themes as th (th.id)}
            <button role="radio" aria-checked={prefs.theme === th.id} class:on={prefs.theme === th.id} onclick={() => setTheme(th.id)}>
              <th.icon size={16} strokeWidth={1.75} />
              {th.label}
            </button>
          {/each}
        </div>
      </div>
      <div class="option">
        <div class="text">
          <span class="title">{t("set.language")}</span>
          <span class="desc">{t("set.languageDesc")}</span>
        </div>
        <div class="segmented" role="radiogroup" aria-label={t("set.language")}>
          {#each languages as lang (lang.id)}
            <button
              role="radio"
              aria-checked={i18n.pref === lang.id}
              class:on={i18n.pref === lang.id}
              onclick={() => setLanguage(lang.id)}>{lang.label}</button
            >
          {/each}
        </div>
      </div>
    </div>
  </section>

  <section>
    <span class="eyebrow">{t("set.general")}</span>
    <div class="group">
      {#each options as opt (opt.key)}
        <div class="option">
          <div class="text">
            <span class="title">{opt.title}</span>
            <span class="desc">{opt.desc}</span>
          </div>
          <Toggle
            state={settings?.[opt.key] ?? false}
            busy={saving === opt.key}
            label={opt.title}
            onchange={() => flip(opt.key)}
          />
        </div>
      {/each}
    </div>
  </section>

  <section>
    <span class="eyebrow">{t("set.updates")}</span>
    <div class="group">
      <div class="option">
        <div class="text">
          <span class="title">{t("set.version", { v: version })}</span>
          <span class="desc">
            {#if !updater.supported}
              {t("upd.packageManaged")}
            {:else if updater.available}
              {t("upd.available", { v: updater.available.version })}
            {:else}
              {t("upd.auto")}
            {/if}
          </span>
        </div>
        {#if !updater.supported}
          <button class="btn" onclick={() => openUrl("https://github.com/Kievy/fontin/releases/latest")}>
            {t("upd.releases")}
          </button>
        {:else if updater.available}
          <button class="btn primary" disabled={updater.progress !== null} onclick={installUpdate}>
            {updater.progress === null ? t("upd.install") : t("upd.downloadingPct", { p: Math.round(updater.progress * 100) })}
          </button>
        {:else}
          <button class="btn" disabled={updater.checking} onclick={() => checkForUpdates(true)}>
            {updater.checking ? t("upd.checking") : t("upd.check")}
          </button>
        {/if}
      </div>
    </div>
  </section>

  <div class="cards">
    <div class="card">
      <span class="eyebrow">{t("set.how")}</span>
      <p>{t("set.howBody")}</p>
    </div>
    <div class="card">
      <span class="eyebrow">{t("set.library")}</span>
      <p class="tabular">
        {t("set.libraryStats", {
          f: data.overview?.counts.all ?? 0,
          a: data.overview?.counts.active ?? 0,
          s: data.overview?.sources.length ?? 0,
        })}
      </p>
    </div>
  </div>
</div>

<style>
  .settings {
    flex: 1;
    overflow-y: auto;
    padding: var(--sp-48);
  }
  .hero {
    display: flex;
    gap: var(--sp-24);
    align-items: center;
  }
  h1 {
    margin: 0;
    font-family: var(--font-display);
    font-size: 40px;
    line-height: 48px;
    font-weight: 700;
    letter-spacing: -0.02em;
  }
  .hero p {
    margin: 0;
    color: var(--text-2);
    max-width: 480px;
  }
  section {
    margin-top: var(--sp-40);
    max-width: 720px;
    display: flex;
    flex-direction: column;
    gap: var(--sp-12);
  }
  .group {
    border-radius: var(--r-16);
    background: var(--raised);
    box-shadow: inset 0 0 0 1px var(--line);
    overflow: hidden;
  }
  .option {
    display: flex;
    align-items: center;
    gap: var(--sp-24);
    padding: var(--sp-16) var(--sp-24);
  }
  .option + .option {
    border-top: 1px solid var(--line);
  }
  .text {
    flex: 1;
    display: flex;
    flex-direction: column;
  }
  .title {
    font-weight: 500;
    color: var(--text);
  }
  .desc {
    font-size: 13px;
    line-height: 16px;
    color: var(--text-3);
  }
  .btn {
    flex: none;
    height: 32px;
    padding: 0 var(--sp-16);
    border-radius: var(--r-8);
    background: var(--raised-2);
    box-shadow: inset 0 0 0 1px var(--line-2);
    font-size: 13px;
    font-weight: 500;
    color: var(--text);
  }
  .btn:hover:not(:disabled) {
    background: var(--press);
  }
  .btn:disabled {
    opacity: 0.6;
    cursor: default;
  }
  .btn.primary {
    background: var(--mint);
    color: var(--ink);
    box-shadow: none;
  }
  .segmented {
    display: flex;
    padding: var(--sp-4);
    gap: var(--sp-4);
    border-radius: var(--r-12);
    background: var(--raised-2);
    box-shadow: inset 0 0 0 1px var(--line);
  }
  .segmented button {
    height: 32px;
    padding: 0 var(--sp-12);
    display: flex;
    align-items: center;
    gap: var(--sp-8);
    border-radius: var(--r-8);
    color: var(--text-3);
    font-size: 13px;
    transition: all 140ms var(--ease);
  }
  .segmented button:hover {
    color: var(--text);
  }
  .segmented button.on {
    background: var(--panel);
    color: var(--text);
    box-shadow:
      inset 0 0 0 1px var(--line-2),
      var(--shadow-sm);
  }
  section + section {
    margin-top: var(--sp-32);
  }
  .cards {
    margin-top: var(--sp-40);
    max-width: 720px;
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(320px, 1fr));
    gap: var(--sp-16);
  }
  .card {
    padding: var(--sp-24);
    border-radius: var(--r-16);
    background: var(--raised);
    box-shadow: inset 0 0 0 1px var(--line);
  }
  .card p {
    margin: var(--sp-8) 0 0;
    color: var(--text-2);
  }
</style>
