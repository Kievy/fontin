<script lang="ts">
  let { size = 40 }: { size?: number } = $props();
</script>

<svg width={size} height={size} viewBox="0 0 64 64" aria-label="Fontin" role="img">
  <rect width="64" height="64" rx="15" fill="#121418" />
  <path d="M29 52V25c0-8 4-12 11-12h3" stroke="#F4F4F0" stroke-width="8" stroke-linecap="round" fill="none" />
  <rect x="15" y="27" width="30" height="12" rx="6" fill="#7DF2A6" />
  <circle cx="39" cy="33" r="3.6" fill="#121418" />
</svg>
