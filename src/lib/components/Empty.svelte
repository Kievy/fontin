<script lang="ts">
  import Logo from "./Logo.svelte";
  import { ui } from "$lib/store.svelte";

  const copy = $derived.by(() => {
    if (ui.search) return { t: "Nada encontrado", d: `Nenhuma família corresponde a “${ui.search}”.` };
    switch (ui.scope.kind) {
      case "all":
        return { t: "Sua biblioteca está vazia", d: "Adicione uma pasta de fontes ou ative algo do Google Fonts — nada é instalado no sistema." };
      case "favorites":
        return { t: "Sem favoritas ainda", d: "Toque no coração de uma família para guardá-la aqui." };
      case "active":
        return { t: "Nenhuma fonte ativa", d: "Ligue o interruptor de uma família para disponibilizá-la em todos os apps." };
      case "collection":
        return { t: "Coleção vazia", d: "Abra uma família e adicione-a a esta coleção pelo painel de detalhes." };
      case "google":
        return { t: "Catálogo indisponível", d: "Verifique sua conexão e use “Atualizar catálogo”." };
      default:
        return { t: "Nada por aqui", d: "Esta visão ainda não tem famílias." };
    }
  });
</script>

<div class="empty">
  <div class="mark"><Logo size={64} /></div>
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
    border-radius: 16px;
    box-shadow:
      0 0 0 1px var(--line-2),
      0 24px 64px -16px var(--mint-glow);
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
    color: var(--text-3);
  }
</style>
