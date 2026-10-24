<script lang="ts">
  interface Props {
    size?: number | string;
    mode?: 'light' | 'dark' | 'auto' | 'brand' | 'white';
    class?: string;
  }

  let {
    size = 24,
    mode = 'brand',
    class: customClass = ''
  }: Props = $props();

  const strokeColor = $derived(
    mode === 'white'
      ? '#FFFFFF'
      : mode === 'light'
        ? '#0D9488'
        : mode === 'brand'
          ? 'url(#cacalmSymbolGrad)'
          : '#2DD4BF'
  );
</script>

<svg
  xmlns="http://www.w3.org/2000/svg"
  viewBox="0 0 512 512"
  width={size}
  height={size}
  class="shrink-0 transition-colors {customClass}"
  aria-label="CACalm Icon"
>
  <defs>
    <linearGradient id="cacalmSymbolGrad" x1="0%" y1="0%" x2="100%" y2="100%">
      <stop offset="0%" stop-color="#5EEAD4" />
      <stop offset="100%" stop-color="#2DD4BF" />
    </linearGradient>
  </defs>

  <!-- Main Symbol Glyph (Transparent CADS standard) -->
  <g fill="none">
    <!-- Pomodoro Circular Arc -->
    <path
      d="M200 120 A 145 145 0 1 1 312 120"
      stroke={strokeColor}
      stroke-width="26"
      stroke-linecap="round"
    />
    <circle cx="200" cy="120" r="14" fill="#FFFFFF" />
    <circle cx="312" cy="120" r="14" fill={mode === 'white' ? '#FFFFFF' : '#2DD4BF'} />

    <!-- Symmetrical Ambient Waves -->
    <path
      d="M165 210 Q 210 180 256 210 T 347 210"
      stroke={strokeColor}
      stroke-width="24"
      stroke-linecap="round"
    />
    <path
      d="M140 256 Q 198 220 256 256 T 372 256"
      stroke={mode === 'light' ? '#0F172A' : '#FFFFFF'}
      stroke-width="26"
      stroke-linecap="round"
    />
    <path
      d="M165 302 Q 210 272 256 302 T 347 302"
      stroke={strokeColor}
      stroke-width="24"
      stroke-linecap="round"
    />

    <!-- Zen Center Ripple Node -->
    <circle cx="256" cy="256" r="11" fill={mode === 'white' ? '#FFFFFF' : '#2DD4BF'} />
  </g>
</svg>
