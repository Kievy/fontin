<script lang="ts">
  // "Adicionar" button: hovering (or clicking/focusing) reveals the actions
  // rising above it with a short stagger.
  import Plus from "@lucide/svelte/icons/plus";
  import FolderPlus from "@lucide/svelte/icons/folder-plus";
  import FilePlus from "@lucide/svelte/icons/file-plus-2";
  import ListPlus from "@lucide/svelte/icons/list-plus";
  import { t } from "$lib/i18n.svelte";

  let {
    onfolder,
    onfont,
    oncollection,
  }: { onfolder: () => void; onfont: () => void; oncollection: () => void } = $props();

  let open = $state(false);
  let closeTimer: ReturnType<typeof setTimeout> | undefined;

  function show() {
    clearTimeout(closeTimer);
    open = true;
  }
  function hide(delay = 160) {
    clearTimeout(closeTimer);
    closeTimer = setTimeout(() => (open = false), delay);
  }
  function run(fn: () => void) {
    open = false;
    fn();
  }

  const items = $derived([
    { label: t("add.folder"), icon: FolderPlus, run: onfolder },
    { label: t("add.font"), icon: FilePlus, run: onfont },
    { label: t("add.collection"), icon: ListPlus, run: oncollection },
  ]);
</script>

<!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
<div
  class="wrap"
  class:open
  role="group"
  onmouseenter={show}
  onmouseleave={() => hide()}
  onfocusin={show}
  onfocusout={(e) => {
    if (!(e.currentTarget as HTMLElement).contains(e.relatedTarget as Node)) hide(0);
  }}
  onkeydown={(e) => e.key === "Escape" && hide(0)}
>
  <div class="menu" role="menu" aria-hidden={!open}>
    {#each items as item, i (item.label)}
      <button
        class="item"
        role="menuitem"
        tabindex={open ? 0 : -1}
        style="--i: {items.length - 1 - i}"
        onclick={() => run(item.run)}
      >
        <span class="ic"><item.icon size={16} strokeWidth={1.75} /></span>
        {item.label}
      </button>
    {/each}
  </div>

  <button class="add" aria-haspopup="menu" aria-expanded={open} onclick={() => (open ? hide(0) : show())}>
    <span class="plus"><Plus size={16} strokeWidth={2} /></span>
    {t("add.button")}
  </button>
</div>

<style>
  .wrap {
    position: relative;
  }
  .menu {
    position: absolute;
    left: 0;
    right: 0;
    bottom: calc(100% + var(--sp-8));
    display: flex;
    flex-direction: column;
    gap: var(--sp-4);
    padding: var(--sp-8);
    border-radius: var(--r-16);
    background: var(--panel);
    box-shadow: var(--shadow-pop);
    opacity: 0;
    transform: translateY(8px) scale(0.98);
    transform-origin: bottom center;
    pointer-events: none;
    transition:
      opacity 160ms var(--ease),
      transform 200ms var(--ease);
    z-index: 30;
  }
  /* Invisible bridge so moving the pointer from the button to the menu keeps it open. */
  .menu::after {
    content: "";
    position: absolute;
    left: 0;
    right: 0;
    top: 100%;
    height: var(--sp-8);
  }
  .open .menu {
    opacity: 1;
    transform: none;
    pointer-events: auto;
  }
  .item {
    height: 40px;
    display: flex;
    align-items: center;
    gap: var(--sp-12);
    padding: 0 var(--sp-12);
    border-radius: var(--r-8);
    color: var(--text-2);
    font-weight: 500;
    text-align: left;
    opacity: 0;
    transform: translateY(8px);
    transition:
      opacity 180ms var(--ease) calc(var(--i) * 40ms),
      transform 220ms var(--ease) calc(var(--i) * 40ms),
      background 120ms var(--ease),
      color 120ms var(--ease);
  }
  .open .item {
    opacity: 1;
    transform: none;
  }
  .item:hover,
  .item:focus-visible {
    background: var(--hover);
    color: var(--text);
  }
  .ic {
    width: 24px;
    height: 24px;
    display: grid;
    place-items: center;
    border-radius: var(--r-8);
    background: var(--raised-2);
    color: var(--text-3);
    transition:
      background 120ms var(--ease),
      color 120ms var(--ease);
  }
  .item:hover .ic {
    background: var(--mint);
    color: var(--ink);
  }
  .add {
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
    font-weight: 500;
    transition: all 160ms var(--ease);
  }
  .add:hover,
  .open .add {
    color: var(--text);
    background: var(--raised-2);
  }
  .plus {
    display: grid;
    place-items: center;
    transition: rotate 220ms var(--ease);
  }
  .open .plus {
    rotate: 45deg;
  }
</style>
