<script lang="ts">
  import Icon from '$lib/components/Icon.svelte';
  import PageHeader from '$lib/components/PageHeader.svelte';
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

  const roundBtn =
    'w-12 h-12 rounded-full flex items-center justify-center border border-[var(--ca-border)] text-[var(--ca-text-secondary)] hover:text-[var(--ca-text-primary)] hover:bg-[var(--ca-surface-subtle)]/60 transition-colors';
</script>

<div class="calm-page">
  <PageHeader icon="timer" title={t('timer.title')} subtitle={t('timer.subtitle')} />

  <div class="grid gap-6 xl:grid-cols-[minmax(0,1fr)_380px] items-start">
    <!-- Timer -->
    <section class="relative overflow-hidden rounded-3xl border border-[var(--ca-border)]/60 bg-[var(--ca-surface)] min-h-[440px] flex items-center">
      <div class="calm-aurora {timer.paused ? '' : 'is-playing'}" aria-hidden="true" style={isBreak ? 'filter: hue-rotate(60deg) blur(10px)' : ''}></div>
      <div class="relative w-full p-8 sm:p-12 flex flex-col lg:flex-row items-center justify-center gap-10 lg:gap-16">
        <div class="relative w-64 h-64 shrink-0">
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

        <div class="flex flex-col items-center lg:items-start gap-5">
          <div class="flex items-center gap-3">
            <button
              type="button"
              onclick={toggleTimer}
              class="flex items-center gap-2 rounded-full px-7 py-3.5 bg-[var(--ca-brand)] text-white font-semibold text-sm shadow-md hover:brightness-110 active:scale-95 transition focus-visible:outline-none focus-visible:ring-4 focus-visible:ring-[var(--ca-brand-glow)]"
            >
              <Icon name={timer.paused ? 'play' : 'pause'} size={16} stroke={0} class="fill-current" />
              {timer.paused ? t('timer.start') : t('timer.pause')}
            </button>
            <button type="button" onclick={resetTimer} class={roundBtn} title="{t('timer.reset')} (Ctrl+R)" aria-label={t('timer.reset')}>
              <Icon name="reset" size={18} />
            </button>
            <button type="button" onclick={skipPhase} class={roundBtn} title="{t('timer.skip')} (Ctrl+Shift+N)" aria-label={t('timer.skip')}>
              <Icon name="skip" size={18} />
            </button>
          </div>
          <label class="flex items-center gap-2.5 text-xs text-[var(--ca-text-secondary)] cursor-pointer w-fit">
            <input type="checkbox" checked={prefs.chime} onchange={(e) => setChime((e.target as HTMLInputElement).checked)} class="accent-[var(--ca-brand)] w-4 h-4" />
            {t('settings.chime')}
          </label>
        </div>
      </div>
    </section>

    <!-- Side panel -->
    <aside class="flex flex-col gap-6">
      <div class="grid grid-cols-2 gap-3">
        <div class="calm-card p-4">
          <div class="text-[11px] text-[var(--ca-text-muted)]">{t('timer.sessionsToday')}</div>
          <div class="text-3xl font-semibold mt-1 tabular-nums">{timer.sessions_today}</div>
        </div>
        <div class="calm-card p-4">
          <div class="text-[11px] text-[var(--ca-text-muted)]">{t('timer.focusToday')}</div>
          <div class="text-3xl font-semibold mt-1 tabular-nums">
            {Math.floor(timer.focus_seconds_today / 60)}
            <span class="text-sm font-normal text-[var(--ca-text-muted)]">{t('timer.minutes')}</span>
          </div>
        </div>
      </div>

      <section class="space-y-2.5">
        <div class="calm-label">{t('timer.pairWithSound')}</div>
        <div class="grid grid-cols-1 sm:grid-cols-3 xl:grid-cols-1 gap-3">
          {#each focusScenes as scene (scene.id)}
            {@const on = mixer.sceneId === scene.id && mixer.playing}
            <button
              type="button"
              onclick={() => applyScene(scene)}
              class="calm-card flex items-center gap-3 text-left p-3.5 transition-colors {on
                ? '!border-[var(--ca-brand)]/50 !bg-[var(--calm-card-active)]'
                : 'hover:!border-[var(--ca-border-hover)]'}"
            >
              <span class="w-10 h-10 shrink-0 rounded-xl flex items-center justify-center {on ? 'bg-[var(--ca-brand)] text-white' : 'bg-[var(--ca-surface-subtle)]/80 text-[var(--ca-brand)]'}">
                <Icon name={scene.icon} size={18} />
              </span>
              <span class="min-w-0">
                <span class="block text-[13px] font-semibold">{loc(scene.name, getLocale())}</span>
                <span class="block text-[11px] text-[var(--ca-text-muted)] truncate">{loc(scene.desc, getLocale())}</span>
              </span>
            </button>
          {/each}
        </div>
      </section>

      <p class="text-[11px] leading-relaxed text-[var(--ca-text-muted)] font-mono">{t('timer.shortcuts')}</p>
    </aside>
  </div>
</div>
