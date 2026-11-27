<script lang="ts">
  import Self from "./FolderTree.svelte";
  import NavItem from "./NavItem.svelte";
  import Toggle from "./Toggle.svelte";
  import Chevron from "@lucide/svelte/icons/chevron-right";
  import { t } from "$lib/i18n.svelte";
  import type { FolderNode } from "$lib/api";
  import { ui, setScope, isScope, locationBusy, locationLevel, toggleLocation } from "$lib/store.svelte";
  import { SvelteSet } from "svelte/reactivity";

  let {
    nodes,
    source,
    depth = 1,
    open,
  }: { nodes: FolderNode[]; source: number; depth?: number; open: SvelteSet<string> } = $props();
</script>

{#each nodes as node (node.path)}
  <NavItem
    {depth}
    count={node.count}
    active={isScope(ui.scope, { kind: "folder", value: node.path })}
    onclick={() => setScope({ kind: "folder", value: node.path })}
  >
    {#snippet icon()}
      {#if node.children.length}
        <span
          class="chev"
          class:open={open.has(node.path)}
          role="button"
          tabindex="-1"
          onclick={(e) => {
            e.stopPropagation();
            if (open.has(node.path)) open.delete(node.path);
            else open.add(node.path);
          }}
          onkeydown={() => {}}
        >
          <Chevron size={14} />
        </span>
      {:else}
        <span class="dot"></span>
      {/if}
    {/snippet}
    {#snippet trailing()}
      {@const level = locationLevel(node.active, node.count)}
      <Toggle
        size="sm"
        state={level}
        busy={locationBusy.has(`${source}:${node.path}`)}
        label={level === 2 ? t("side.deactivateFolder") : t("side.activateFolder")}
        onchange={() => toggleLocation(source, node.path, level)}
      />
    {/snippet}
    {node.name}
  </NavItem>
  {#if node.children.length && open.has(node.path)}
    <Self nodes={node.children} {source} depth={depth + 1} {open} />
  {/if}
{/each}

<style>
  .chev {
    display: grid;
    place-items: center;
    transition: transform 160ms var(--ease);
  }
  .chev.open {
    transform: rotate(90deg);
  }
  .dot {
    width: 4px;
    height: 4px;
    border-radius: 50%;
    background: var(--text-4);
  }
</style>
