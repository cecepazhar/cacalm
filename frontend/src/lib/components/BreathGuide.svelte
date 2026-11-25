<script lang="ts">
  import { onMount } from 'svelte';
  import { engine } from '$lib/audio/engine';
  import { t } from '$lib/i18n/index.svelte';

  interface Props {
    id: string;
    phases: [number, number, number, number];
  }

  let { id, phases }: Props = $props();

  let label = $state('');
  let count = $state(0);
  let scale = $state(0.6);

  onMount(() => {
    let raf = 0;
    const frame = () => {
      const pos = engine.loopPosition(id);
      if (pos !== null) {
        const [inh, h1, exh, h2] = phases;
        let p = pos;
        if (p < inh) {
          label = t('sounds.inhale');
          count = Math.ceil(inh - p);
          scale = 0.6 + 0.4 * (0.5 - 0.5 * Math.cos((Math.PI * p) / inh));
        } else if ((p -= inh) < h1) {
          label = t('sounds.hold');
          count = Math.ceil(h1 - p);
          scale = 1;
        } else if ((p -= h1) < exh) {
          label = t('sounds.exhale');
          count = Math.ceil(exh - p);
          scale = 0.6 + 0.4 * (0.5 + 0.5 * Math.cos((Math.PI * p) / exh));
        } else {
          p -= exh;
          label = t('sounds.hold');
          count = Math.ceil(h2 - p);
          scale = 0.6;
        }
      }
      raf = requestAnimationFrame(frame);
    };
    raf = requestAnimationFrame(frame);
    return () => cancelAnimationFrame(raf);
  });
</script>

<div class="flex items-center gap-4">
  <div class="relative w-20 h-20 flex items-center justify-center">
    <div
      class="absolute inset-0 rounded-full bg-[var(--ca-brand)]/20 border border-[var(--ca-brand)]/40"
      style="transform: scale({scale}); transition: transform 120ms linear;"
    ></div>
    <span class="relative font-mono text-lg font-semibold tabular-nums">{count}</span>
  </div>
  <div class="text-sm font-semibold">{label}</div>
</div>
