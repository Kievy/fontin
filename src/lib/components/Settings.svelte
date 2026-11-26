<script lang="ts">
  import Logo from "./Logo.svelte";
  import { data } from "$lib/store.svelte";
</script>

<div class="settings">
  <div class="hero">
    <Logo size={64} />
    <div>
      <h1>fontin</h1>
      <p>Gerenciador de fontes open source. A fonte entra no sistema sem ser instalada: liga e desliga.</p>
    </div>
  </div>
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
  .cards {
    margin-top: var(--sp-40);
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
