<script lang="ts">
  import Icon from '$lib/components/Icon.svelte';
  import { t, getLocale } from '$lib/i18n/index.svelte';
  import { SCENES, loc } from '$lib/audio/catalog';
  import { getMixer, applyScene } from '$lib/stores/mixer.svelte';
  import {
    getTimer,
    getTimerPrefs,
    setChime,
    toggleTimer,
    resetTimer,
    skipPhase,
    phaseProgress,
    formatClock,
  } from '$lib/stores/timer.svelte';

  const timer = getTimer();
  const prefs = getTimerPrefs();
  const mixer = getMixer();

  const R = 108;
  const C = 2 * Math.PI * R;
  const progress = $derived.by(() => {
    void timer.remaining;
    return phaseProgress();
  });

  const phaseLabel = $derived(
    timer.phase === 'FOCUS' ? t('timer.focus') : timer.phase === 'LONG BREAK' ? t('timer.longBreak') : t('timer.shortBreak')
  );
  const isBreak = $derived(timer.phase !== 'FOCUS');
  const cycleDot = $derived(timer.sessions_today % 4);

  const focusScenes = SCENES.filter((s) => ['sc-focus', 'sc-cafe', 'sc-calm'].includes(s.id));
</script>

<div class="max-w-4xl mx-auto w-full px-4 sm:px-6 py-6 space-y-6">
  <div>
    <h1 class="text-xl font-semibold tracking-tight">{t('timer.title')}</h1>
    <p class="text-sm text-[var(--ca-text-muted)] mt-1">{t('timer.subtitle')}</p>
  </div>

  <section class="relative overflow-hidden rounded-3xl border border-[var(--ca-border)]/60 bg-[var(--ca-surface)] p-6 sm:p-10">
    <div class="calm-aurora {timer.paused ? '' : 'is-playing'}" aria-hidden="true" style={isBreak ? 'filter: hue-rotate(60deg) blur(10px)' : ''}></div>
    <div class="relative flex flex-col md:flex-row items-center gap-8 md:gap-12">
      <!-- Ring -->
      <div class="relative w-60 h-60 shrink-0">
        <svg viewBox="0 0 240 240" class="w-full h-full -rotate-90">
          <circle cx="120" cy="120" r={R} fill="none" stroke="var(--ca-surface-subtle)" stroke-width="10" />
          <circle
            cx="120"
            cy="120"
            r={R}
            fill="none"
            stroke={isBreak ? '#818cf8' : 'var(--ca-brand)'}
            stroke-width="10"
            stroke-linecap="round"
            stroke-dasharray={C}
            stroke-dashoffset={C * (1 - progress)}
            style="transition: stroke-dashoffset 0.9s linear"
          />
        </svg>
        <div class="absolute inset-0 flex flex-col items-center justify-center">
          <span class="text-[11px] uppercase tracking-[0.2em] font-semibold {isBreak ? 'text-indigo-400' : 'text-[var(--ca-brand)]'}">
            {phaseLabel}
          </span>
          <span class="text-5xl font-light font-mono tabular-nums tracking-tight mt-1">{formatClock(timer.remaining)}</span>
          <div class="flex gap-1.5 mt-3" aria-hidden="true">
            {#each [0, 1, 2, 3] as i (i)}
              <span class="w-1.5 h-1.5 rounded-full {i < cycleDot ? 'bg-[var(--ca-brand)]' : 'bg-[var(--ca-surface-subtle)]'}"></span>
            {/each}
          </div>
        </div>
      </div>

      <!-- Controls + stats -->
      <div class="flex-1 w-full space-y-6">
        <div class="flex items-center gap-3">
          <button
            type="button"
            onclick={toggleTimer}
            class="flex items-center gap-2 rounded-full px-6 py-3 bg-[var(--ca-brand)] text-white font-semibold text-sm shadow-md hover:brightness-110 active:scale-95 transition focus-visible:outline-none focus-visible:ring-4 focus-visible:ring-[var(--ca-brand-glow)]"
          >
            <Icon name={timer.paused ? 'play' : 'pause'} size={16} stroke={0} class="fill-current" />
            {timer.paused ? t('timer.start') : t('timer.pause')}
          </button>
          <button
            type="button"
            onclick={resetTimer}
            class="w-11 h-11 rounded-full flex items-center justify-center border border-[var(--ca-border)] text-[var(--ca-text-secondary)] hover:text-[var(--ca-text-primary)] hover:bg-[var(--ca-surface-subtle)]/60"
            title="{t('timer.reset')} (Ctrl+R)"
            aria-label={t('timer.reset')}
          >
            <Icon name="reset" size={17} />
          </button>
          <button
            type="button"
            onclick={skipPhase}
            class="w-11 h-11 rounded-full flex items-center justify-center border border-[var(--ca-border)] text-[var(--ca-text-secondary)] hover:text-[var(--ca-text-primary)] hover:bg-[var(--ca-surface-subtle)]/60"
            title="{t('timer.skip')} (Ctrl+Shift+N)"
            aria-label={t('timer.skip')}
          >
            <Icon name="skip" size={17} />
          </button>
        </div>

        <div class="grid grid-cols-2 gap-3">
          <div class="rounded-2xl bg-[var(--ca-surface)]/70 border border-[var(--ca-border)]/60 p-4">
            <div class="text-[11px] text-[var(--ca-text-muted)]">{t('timer.sessionsToday')}</div>
            <div class="text-2xl font-semibold mt-1 tabular-nums">{timer.sessions_today}</div>
          </div>
          <div class="rounded-2xl bg-[var(--ca-surface)]/70 border border-[var(--ca-border)]/60 p-4">
            <div class="text-[11px] text-[var(--ca-text-muted)]">{t('timer.focusToday')}</div>
            <div class="text-2xl font-semibold mt-1 tabular-nums">
              {Math.floor(timer.focus_seconds_today / 60)}
              <span class="text-sm font-normal text-[var(--ca-text-muted)]">{t('timer.minutes')}</span>
            </div>
          </div>
        </div>

        <label class="flex items-center gap-2.5 text-xs text-[var(--ca-text-secondary)] cursor-pointer w-fit">
          <input type="checkbox" checked={prefs.chime} onchange={(e) => setChime((e.target as HTMLInputElement).checked)} class="accent-[var(--ca-brand)] w-4 h-4" />
          {t('settings.chime')}
        </label>
      </div>
    </div>
  </section>

  <section class="space-y-2.5">
    <div class="text-[11px] uppercase tracking-wider font-semibold text-[var(--ca-text-muted)]">{t('timer.pairWithSound')}</div>
    <div class="grid grid-cols-1 sm:grid-cols-3 gap-3">
      {#each focusScenes as scene (scene.id)}
        {@const on = mixer.sceneId === scene.id && mixer.playing}
        <button
          type="button"
          onclick={() => applyScene(scene)}
          class="flex items-center gap-3 text-left rounded-2xl border p-3.5 transition-colors {on
            ? 'border-[var(--ca-brand)]/50 bg-[var(--calm-card-active)]'
            : 'border-[var(--ca-border)]/70 bg-[var(--ca-surface)] hover:border-[var(--ca-border-hover)]'}"
        >
          <span class="w-9 h-9 rounded-xl flex items-center justify-center {on ? 'bg-[var(--ca-brand)] text-white' : 'bg-[var(--ca-surface-subtle)]/80 text-[var(--ca-brand)]'}">
            <Icon name={scene.icon} size={17} />
          </span>
          <span class="min-w-0">
            <span class="block text-[13px] font-semibold">{loc(scene.name, getLocale())}</span>
            <span class="block text-[11px] text-[var(--ca-text-muted)] truncate">{loc(scene.desc, getLocale())}</span>
          </span>
        </button>
      {/each}
    </div>
  </section>

  <p class="text-[11px] text-[var(--ca-text-muted)] font-mono">{t('timer.shortcuts')}</p>
</div>
