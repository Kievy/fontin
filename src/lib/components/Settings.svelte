<script lang="ts">
  import Logo from "./Logo.svelte";
  import Toggle from "./Toggle.svelte";
  import { onMount } from "svelte";
  import { api, type AppSettings } from "$lib/api";
  import { data, toast, prefs, setTheme } from "$lib/store.svelte";
  import type { ThemePref } from "$lib/theme";
  import Monitor from "@lucide/svelte/icons/monitor";
  import Sun from "@lucide/svelte/icons/sun";
  import Moon from "@lucide/svelte/icons/moon";

  const themes: { id: ThemePref; label: string; icon: typeof Sun }[] = [
    { id: "system", label: "Sistema", icon: Monitor },
    { id: "light", label: "Claro", icon: Sun },
    { id: "dark", label: "Escuro", icon: Moon },
  ];

  let settings = $state<AppSettings | null>(null);
  let saving = $state<keyof AppSettings | null>(null);

  onMount(async () => {
    try {
      settings = await api.getSettings();
    } catch (e) {
      toast(String(e), "error");
    }
  });

  const options: { key: keyof AppSettings; title: string; desc: string }[] = [
    {
      key: "hideOnClose",
      title: "Ocultar ao fechar",
      desc: "Fechar a janela mantém o Fontin na bandeja do sistema, com as fontes ativas.",
    },
    {
      key: "autostart",
      title: "Iniciar ao ligar o sistema",
      desc: "Abre o Fontin quando você entra na sua sessão, reativando suas fontes.",
    },
    {
      key: "startHidden",
      title: "Iniciar oculto",
      desc: "Abre direto na bandeja, sem mostrar a janela. Clique no ícone para abrir.",
    },
    {
      key: "deactivateOnExit",
      title: "Desativar todas as fontes ao encerrar",
      desc: "Ao sair, remove as fontes do sistema. Elas voltam a ser ativadas na próxima abertura.",
    },
  ];

  async function flip(key: keyof AppSettings) {
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
      <p>Gerenciador de fontes open source. A fonte entra no sistema sem ser instalada: liga e desliga.</p>
    </div>
  </div>

  <section>
    <span class="eyebrow">Aparência</span>
    <div class="group">
      <div class="option">
        <div class="text">
          <span class="title">Tema</span>
          <span class="desc">“Sistema” acompanha o modo claro/escuro do seu sistema operacional.</span>
        </div>
        <div class="segmented" role="radiogroup" aria-label="Tema">
          {#each themes as t (t.id)}
            <button role="radio" aria-checked={prefs.theme === t.id} class:on={prefs.theme === t.id} onclick={() => setTheme(t.id)}>
              <t.icon size={16} strokeWidth={1.75} />
              {t.label}
            </button>
          {/each}
        </div>
      </div>
    </div>
  </section>

  <section>
    <span class="eyebrow">Geral</span>
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

  <div class="cards">
    <div class="card">
      <span class="eyebrow">Como funciona</span>
      <p>
        Ativar cria apenas uma referência temporária para o arquivo original — no Linux via fontconfig, no Windows por
        sessão (GDI) e no macOS pelo CoreText. Desativar remove a referência; seus arquivos nunca são movidos ou
        copiados.
      </p>
    </div>
    <div class="card">
      <span class="eyebrow">Biblioteca</span>
      <p class="tabular">
        {data.overview?.counts.all.toLocaleString("pt-BR") ?? 0} famílias ·
        {data.overview?.counts.active.toLocaleString("pt-BR") ?? 0} ativas ·
        {data.overview?.sources.length ?? 0} pastas
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
