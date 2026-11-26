<script lang="ts">
  // The brand switch: the bar of the "f" in the logo, on = mint with a dark knob.
  let {
    state = 0,
    busy = false,
    progress = null,
    size = "md",
    label = "Ativar",
    onchange,
  }: {
    state?: 0 | 1 | 2 | boolean;
    busy?: boolean;
    progress?: number | null;
    size?: "sm" | "md";
    label?: string;
    onchange?: () => void;
  } = $props();

  const level = $derived(state === true ? 2 : state === false ? 0 : state);
</script>

<button
  class="toggle {size}"
  class:on={level === 2}
  class:partial={level === 1}
  class:busy
  role="switch"
  aria-checked={level === 2 ? "true" : level === 1 ? "mixed" : "false"}
  aria-label={label}
  title={label}
  onclick={(e) => {
    e.stopPropagation();
    onchange?.();
  }}
>
  {#if progress !== null}
    <span class="fill" style="width: {Math.max(8, progress * 100)}%"></span>
  {/if}
  <span class="knob"></span>
</button>

<style>
  .toggle {
    --w: 40px;
    --h: 24px;
    --k: 8px;
    position: relative;
    flex: none;
    width: var(--w);
    height: var(--h);
    border-radius: var(--r-full);
    background: var(--raised-2);
    box-shadow: inset 0 0 0 1px var(--line-2);
    overflow: hidden;
    transition:
      background 180ms var(--ease),
      box-shadow 180ms var(--ease);
  }
  .toggle.sm {
    --w: 32px;
    --h: 16px;
    --k: 6px;
  }
  .toggle:hover {
    box-shadow: inset 0 0 0 1px var(--line-strong);
  }
  .knob {
    position: absolute;
    top: 50%;
    left: calc(var(--h) / 2);
    width: var(--k);
    height: var(--k);
    margin: calc(var(--k) / -2) 0 0 calc(var(--k) / -2);
    border-radius: 50%;
    background: var(--text-3);
    transition:
      left 220ms var(--ease),
      background 180ms var(--ease),
      transform 180ms var(--ease);
  }
  .toggle.on {
    background: var(--mint);
    box-shadow:
      inset 0 0 0 1px rgba(18, 20, 24, 0.08),
      0 0 16px -4px var(--mint-glow);
  }
  .toggle.on .knob {
    left: calc(var(--w) - var(--h) / 2);
    background: var(--ink);
    transform: scale(1.25);
  }
  .toggle.partial {
    background: var(--mint-soft);
    box-shadow: inset 0 0 0 1px var(--accent-line);
  }
  .toggle.partial .knob {
    left: 50%;
    background: var(--accent-text);
  }
  .toggle.busy .knob {
    animation: pulse 900ms ease-in-out infinite;
  }
  .fill {
    position: absolute;
    inset: 0 auto 0 0;
    background: var(--mint-soft);
    transition: width 200ms var(--ease);
  }
  @keyframes pulse {
    50% {
      transform: scale(0.6);
      opacity: 0.5;
    }
  }
</style>
