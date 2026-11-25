<script lang="ts">
  import { onMount } from 'svelte';
  import Icon from './Icon.svelte';
  import { t, getLocale } from '$lib/i18n/index.svelte';
  import { loc } from '$lib/audio/catalog';
  import { engine } from '$lib/audio/engine';
  import { createRenderer, type Renderer } from '$lib/visuals/renderers';
  import type { ModeDef } from '$lib/modes/catalog';
  import { closeMode } from '$lib/stores/modes.svelte';
  import {
    getMixer,
    applyModeMix,
    togglePlay,
    setMaster,
    setSleepTimer,
    setSleepAction,
    sleepRemaining,
    type SleepAction,
  } from '$lib/stores/mixer.svelte';
  import { formatClock } from '$lib/stores/timer.svelte';
  import { isTauri } from '$lib/env';

  interface Props {
    mode: ModeDef;
  }
  let { mode }: Props = $props();

  const mixer = getMixer();
  const SLEEP_OPTIONS = [0, 15, 30, 45, 60, 90];
  const ACTIONS: SleepAction[] = ['pause', 'quit', 'sleep', 'shutdown'];

  let canvasEl: HTMLCanvasElement;
  let controlsVisible = $state(true);
  let fullscreen = $state(true);
  let dim = $state(0);
  let clock = $state('');
  let napMinutes = $state(0);
  let napEndsAt = $state<number | null>(null);
  let napLeft = $state(0);
  let waking = $state(false);
  let renderer: Renderer | null = null;
  const startedAt = Date.now();

  async function setFullscreen(on: boolean) {
    fullscreen = on;
    try {
      if (isTauri()) {
        const { invoke } = await import('@tauri-apps/api/core');
        await invoke('window_set_fullscreen', { fullscreen: on });
      } else if (on && !document.fullscreenElement) {
        await document.documentElement.requestFullscreen?.();
      } else if (!on && document.fullscreenElement) {
        await document.exitFullscreen?.();
      }
    } catch {
      /* not supported */
    }
  }

  function exit() {
    void setFullscreen(false);
    closeMode();
  }

  function startNap(minutes: number) {
    napMinutes = minutes;
    waking = false;
    renderer?.set?.('wake', 0);
    napEndsAt = Date.now() + minutes * 60_000;
    applyModeMix(mode.id, mode.mix);
  }

  function wakeUp() {
    if (!mode.nap || waking) return;
    waking = true;
    napEndsAt = null;
    applyModeMix(mode.id, mode.nap.wakeMix);
    [0, 6000, 12000].forEach((d) => setTimeout(() => engine.chime(0.3), d));
    const t0 = performance.now();
    const ramp = () => {
      const k = Math.min(1, (performance.now() - t0) / 20000);
      renderer?.set?.('wake', k);
      if (k < 1) setTimeout(ramp, 250);
    };
    ramp();
  }

  // Auto-hide controls (and the cursor) after 3 s without input.
  let hideTimer: ReturnType<typeof setTimeout> | null = null;
  function poke() {
    controlsVisible = true;
    if (hideTimer) clearTimeout(hideTimer);
    hideTimer = setTimeout(() => (controlsVisible = false), 3000);
  }

  function onKey(e: KeyboardEvent) {
    if (e.key === 'Escape' && mixer.powerCountdown === null) {
      e.preventDefault();
      exit();
    }
    poke();
  }

  onMount(() => {
    void setFullscreen(true);
    if (mode.nap) startNap(mode.nap.choices[0]);
    else applyModeMix(mode.id, mode.mix);
    if (mode.sleep) setSleepTimer(mode.sleep.minutes, mode.sleep.action);

    const ctx = canvasEl.getContext('2d', { alpha: false })!;
    renderer = createRenderer(mode.renderer);
    const size = () => {
      // Render below screen resolution (per-mode budget); CSS upscales smoothly.
      const scale = Math.min(window.devicePixelRatio || 1, 1.25) * renderer!.quality.scale;
      canvasEl.width = Math.round(window.innerWidth * scale);
      canvasEl.height = Math.round(window.innerHeight * scale);
      ctx.setTransform(1, 0, 0, 1, 0, 0);
      renderer!.resize(canvasEl.width, canvasEl.height);
    };
    size();
    window.addEventListener('resize', size);

    // Frame loop capped at the renderer's fps; stops drawing when the window is in the background
    // (html.calm-idle) or when the night mode has fully dimmed.
    let raf = 0;
    let last = performance.now();
    let acc = 0;
    const t0 = performance.now();
    const loop = (now: number) => {
      raf = requestAnimationFrame(loop);
      const dt = (now - last) / 1000;
      last = now;
      acc += dt;
      if (acc < 1 / renderer!.quality.fps) return;
      const step = Math.min(acc, 0.1);
      acc = 0;
      if (document.documentElement.classList.contains('calm-idle') || dim >= 0.97) return;
      renderer!.frame(ctx, (now - t0) / 1000, step);
    };
    raf = requestAnimationFrame(loop);

    // Once-a-second housekeeping: clock, dimming, nap countdown.
    const tick = () => {
      const d = new Date();
      clock = d.toLocaleTimeString(getLocale() === 'id' ? 'id-ID' : 'en-US', { hour: '2-digit', minute: '2-digit' });
      if (mode.dim) {
        const min = (Date.now() - startedAt) / 60000;
        dim = Math.max(0, Math.min(0.97, ((min - mode.dim.afterMin) / mode.dim.overMin) * 0.97));
      }
      if (napEndsAt) {
        napLeft = Math.max(0, Math.ceil((napEndsAt - Date.now()) / 1000));
        if (napLeft <= 0) wakeUp();
      }
    };
    tick();
    const tickTimer = setInterval(tick, 1000);
    poke();

    return () => {
      cancelAnimationFrame(raf);
      clearInterval(tickTimer);
      if (hideTimer) clearTimeout(hideTimer);
      window.removeEventListener('resize', size);
    };
  });

  const sleepLeft = $derived.by(() => {
    void mixer.now;
    return sleepRemaining();
  });

  const pill =
    'flex items-center gap-2 rounded-full bg-black/45 border border-white/10 px-3 py-2 text-xs text-white/85';
  const iconBtn =
    'w-10 h-10 rounded-full flex items-center justify-center bg-black/45 border border-white/10 text-white/85 hover:text-white hover:bg-black/60 transition-colors';
</script>

<svelte:window onkeydown={onKey} />

<!-- Pointer activity only reveals the controls; keyboard is handled on window. -->
<!-- svelte-ignore a11y_click_events_have_key_events -->
<div
  tabindex="-1"
  class="fixed inset-0 z-[70] bg-black select-none"
  style:cursor={controlsVisible ? 'default' : 'none'}
  onmousemove={poke}
  onclick={poke}
  ontouchstart={poke}
  role="dialog"
  aria-modal="true"
  aria-label={loc(mode.name, getLocale())}
>
  <canvas bind:this={canvasEl} class="absolute inset-0 w-full h-full"></canvas>
  <div class="absolute inset-0 bg-black pointer-events-none transition-opacity duration-1000" style:opacity={dim}></div>

  <!-- Top: title + clock -->
  <div class="absolute top-0 inset-x-0 p-5 flex items-start justify-between transition-opacity duration-500 {controlsVisible ? 'opacity-100' : 'opacity-0 pointer-events-none'}">
    <div class="{pill} !px-4">
      <Icon name={mode.icon} size={15} />
      <span class="font-semibold">{loc(mode.name, getLocale())}</span>
      <span class="text-white/50">·</span>
      <span class="font-mono tabular-nums">{clock}</span>
    </div>
    <div class="flex items-center gap-2">
      <button type="button" class={iconBtn} onclick={() => setFullscreen(!fullscreen)} title={fullscreen ? t('mode.window') : t('mode.fullscreen')} aria-label={fullscreen ? t('mode.window') : t('mode.fullscreen')}>
        <Icon name={fullscreen ? 'restore' : 'maximize'} size={15} />
      </button>
      <button type="button" class={iconBtn} onclick={exit} title="{t('mode.exit')} (Esc)" aria-label={t('mode.exit')}>
        <Icon name="x" size={17} />
      </button>
    </div>
  </div>

  {#if mode.nap && (napEndsAt || waking)}
    <div class="absolute inset-x-0 top-[38%] text-center pointer-events-none transition-opacity duration-700 {controlsVisible || waking ? 'opacity-100' : 'opacity-40'}">
      <p class="text-sm uppercase tracking-[0.3em] text-slate-700/80">{waking ? t('mode.wakeUp') : t('mode.napLeft')}</p>
      {#if !waking}<p class="text-6xl font-light font-mono tabular-nums text-slate-800/80 mt-2">{formatClock(napLeft)}</p>{/if}
    </div>
  {/if}

  <!-- Bottom controls -->
  <div class="absolute bottom-0 inset-x-0 p-6 flex flex-wrap items-center justify-center gap-3 transition-opacity duration-500 {controlsVisible ? 'opacity-100' : 'opacity-0 pointer-events-none'}">
    <button
      type="button"
      onclick={togglePlay}
      class="w-14 h-14 rounded-full flex items-center justify-center bg-white/90 text-black hover:bg-white transition-colors"
      aria-label={mixer.playing ? t('sounds.pause') : t('sounds.play')}
    >
      <Icon name={mixer.playing ? 'pause' : 'play'} size={20} stroke={0} class="fill-current" />
    </button>

    <div class="{pill} w-44">
      <Icon name="volume" size={15} />
      <input
        type="range"
        min="0"
        max="1"
        step="0.01"
        value={mixer.master}
        oninput={(e) => setMaster(Number((e.target as HTMLInputElement).value))}
        class="calm-range"
        style="--val:{Math.round(mixer.master * 100)}"
        aria-label={t('master.volume')}
      />
    </div>

    {#if mode.nap}
      <div class={pill}>
        <Icon name="timer" size={15} />
        {#each mode.nap.choices as m (m)}
          <button type="button" onclick={() => startNap(m)} class="rounded-full px-2.5 py-0.5 {napMinutes === m && !waking ? 'bg-white text-black font-semibold' : 'hover:bg-white/10'}">
            {t('sleep.minutes', { n: m })}
          </button>
        {/each}
      </div>
    {:else}
      <div class={pill}>
        <Icon name="moon" size={15} />
        <select
          class="bg-transparent outline-none cursor-pointer"
          value={mixer.sleepEndsAt ? mixer.sleepMinutes : 0}
          onchange={(e) => setSleepTimer(Number((e.target as HTMLSelectElement).value))}
          aria-label={t('sleep.title')}
        >
          {#each SLEEP_OPTIONS as m (m)}
            <option value={m} class="text-black">{m === 0 ? t('sleep.off') : t('sleep.minutes', { n: m })}</option>
          {/each}
        </select>
        {#if mixer.sleepEndsAt}<span class="font-mono tabular-nums text-white/60">{formatClock(sleepLeft)}</span>{/if}
        <span class="text-white/30">|</span>
        <select
          class="bg-transparent outline-none cursor-pointer"
          value={mixer.sleepAction}
          onchange={(e) => setSleepAction((e.target as HTMLSelectElement).value as SleepAction)}
          aria-label={t('sleep.whenDone')}
        >
          {#each ACTIONS as a (a)}
            <option value={a} class="text-black">{t('sleep.action.' + a)}</option>
          {/each}
        </select>
      </div>
    {/if}
  </div>
</div>
