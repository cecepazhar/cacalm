<script lang="ts">
  import Icon from './Icon.svelte';
  import { t, getLocale } from '$lib/i18n/index.svelte';
  import { SOUND_BY_ID, SCENES, loc } from '$lib/audio/catalog';
  import {
    getMixer,
    togglePlay,
    setMaster,
    toggleMasterMute,
    setSleepTimer,
    setSleepAction,
    type SleepAction,
    sleepRemaining,
    nameOfCustom,
  } from '$lib/stores/mixer.svelte';
  import { getTimer, formatClock } from '$lib/stores/timer.svelte';

  const mixer = getMixer();
  const timer = getTimer();

  let sleepOpen = $state(false);
  const SLEEP_OPTIONS = [0, 15, 30, 45, 60, 90];
  const ACTIONS: { id: SleepAction; icon: string }[] = [
    { id: 'pause', icon: 'pause' },
    { id: 'quit', icon: 'x' },
    { id: 'sleep', icon: 'moon' },
    { id: 'shutdown', icon: 'power' },
  ];

  const title = $derived.by(() => {
    if (mixer.selected.length === 0) return t('sounds.nothing');
    const scene = SCENES.find((s) => s.id === mixer.sceneId);
    if (scene) return loc(scene.name, getLocale());
    const user = mixer.userMixes.find((m) => m.id === mixer.sceneId);
    if (user) return user.name;
    return mixer.selected
      .slice(0, 3)
      .map((id) => (SOUND_BY_ID.get(id) ? loc(SOUND_BY_ID.get(id)!.name, getLocale()) : nameOfCustom(id)))
      .join(' · ') + (mixer.selected.length > 3 ? ` +${mixer.selected.length - 3}` : '');
  });

  const anyLoading = $derived(mixer.selected.some((id) => mixer.loading[id]));
  const sleepLeft = $derived.by(() => {
    void mixer.now;
    return sleepRemaining();
  });
  const masterVal = $derived(mixer.masterMuted ? 0 : mixer.master);
</script>

<svelte:window onclick={() => (sleepOpen = false)} />

<div class="shrink-0 h-[68px] border-t border-[var(--ca-border)]/70 bg-[var(--ca-surface)] px-3 sm:px-5 flex items-center gap-3 sm:gap-5 relative z-20">
  <!-- Play / pause -->
  <div class="relative shrink-0">
    {#if mixer.playing}
      <span class="absolute inset-0 rounded-full bg-[var(--ca-brand)] calm-breathe-ring" aria-hidden="true"></span>
    {/if}
    <button
      type="button"
      onclick={togglePlay}
      disabled={mixer.selected.length === 0}
      class="relative w-11 h-11 rounded-full flex items-center justify-center bg-[var(--ca-brand)] text-white shadow-md hover:brightness-110 active:scale-95 transition disabled:opacity-40 disabled:cursor-default focus-visible:outline-none focus-visible:ring-4 focus-visible:ring-[var(--ca-brand-glow)]"
      title={mixer.playing ? t('sounds.pause') : t('sounds.play')}
      aria-label={mixer.playing ? t('sounds.pause') : t('sounds.play')}
    >
      {#if anyLoading && mixer.playing}
        <span class="w-4 h-4 rounded-full border-2 border-white/40 border-t-white calm-spin"></span>
      {:else}
        <Icon name={mixer.playing ? 'pause' : 'play'} size={18} stroke={0} class="fill-current" />
      {/if}
    </button>
  </div>

  <!-- Now playing -->
  <div class="min-w-0 flex-1">
    <div class="text-[10px] uppercase tracking-wider font-semibold text-[var(--ca-text-muted)] flex items-center gap-2">
      {mixer.playing ? t('sounds.nowPlaying') : t('sounds.paused')}
      {#if mixer.playing}
        <span class="calm-eq" aria-hidden="true"><span></span><span></span><span></span></span>
      {/if}
    </div>
    <div class="text-[13px] font-medium truncate">{title}</div>
  </div>

  <!-- Focus timer chip -->
  {#if !timer.paused}
    <a
      href="/focus"
      class="hidden md:flex items-center gap-2 rounded-full px-3 py-1.5 bg-[var(--ca-surface-subtle)]/70 text-xs font-medium hover:bg-[var(--ca-surface-subtle)]"
    >
      <Icon name="timer" size={14} class="text-[var(--ca-brand)]" />
      <span class="font-mono tabular-nums">{formatClock(timer.remaining)}</span>
      <span class="text-[var(--ca-text-muted)]">
        {timer.phase === 'FOCUS' ? t('timer.focus') : timer.phase === 'LONG BREAK' ? t('timer.longBreak') : t('timer.shortBreak')}
      </span>
    </a>
  {/if}

  <!-- Sleep timer -->
  <div class="relative">
    <button
      type="button"
      onclick={(e) => {
        e.stopPropagation();
        sleepOpen = !sleepOpen;
      }}
      class="flex items-center gap-1.5 rounded-full px-3 py-1.5 text-xs font-medium transition-colors {mixer.sleepEndsAt
        ? 'bg-indigo-500/15 text-indigo-400'
        : 'text-[var(--ca-text-secondary)] hover:bg-[var(--ca-surface-subtle)]/70'}"
      title={t('sleep.title')}
      aria-label={t('sleep.title')}
      aria-expanded={sleepOpen}
    >
      <Icon name="moon" size={15} />
      {#if mixer.sleepEndsAt}
        <span class="font-mono tabular-nums">{mixer.sleepFading ? t('sleep.fading') : formatClock(sleepLeft)}</span>
      {:else}
        <span class="hidden sm:inline">{t('sleep.title')}</span>
      {/if}
    </button>
    {#if sleepOpen}
      <!-- svelte-ignore a11y_click_events_have_key_events, a11y_no_static_element_interactions -->
      <div
        class="absolute bottom-12 right-0 w-64 rounded-2xl border border-[var(--ca-border)] bg-[var(--ca-surface)] shadow-xl p-2"
        onclick={(e) => e.stopPropagation()}
      >
        <div class="px-2 pt-1 pb-2 text-[11px] font-semibold text-[var(--ca-text-muted)] uppercase tracking-wider">
          {t('sleep.title')}
        </div>
        <div class="grid grid-cols-3 gap-1">
          {#each SLEEP_OPTIONS as m (m)}
            <button
              type="button"
              onclick={() => {
                setSleepTimer(m);
                sleepOpen = false;
              }}
              class="rounded-lg py-1.5 text-xs font-medium transition-colors {mixer.sleepMinutes === m && (m === 0 ? !mixer.sleepEndsAt : !!mixer.sleepEndsAt)
                ? 'bg-[var(--ca-brand)] text-white'
                : 'hover:bg-[var(--ca-surface-subtle)] text-[var(--ca-text-secondary)]'}"
            >
              {m === 0 ? t('sleep.off') : t('sleep.minutes', { n: m })}
            </button>
          {/each}
        </div>
        <div class="px-2 pt-3 pb-1.5 text-[11px] font-semibold text-[var(--ca-text-muted)] uppercase tracking-wider">
          {t('sleep.whenDone')}
        </div>
        <div class="space-y-0.5">
          {#each ACTIONS as a (a.id)}
            <button
              type="button"
              onclick={() => setSleepAction(a.id)}
              class="w-full flex items-center gap-2.5 rounded-lg px-2.5 py-1.5 text-xs text-left transition-colors {mixer.sleepAction === a.id
                ? 'bg-[var(--ca-brand)]/12 text-[var(--ca-brand)] font-semibold'
                : 'hover:bg-[var(--ca-surface-subtle)] text-[var(--ca-text-secondary)]'}"
              aria-pressed={mixer.sleepAction === a.id}
            >
              <Icon name={a.icon} size={14} />
              <span class="flex-1">{t('sleep.action.' + a.id)}</span>
              {#if mixer.sleepAction === a.id}<Icon name="check" size={13} />{/if}
            </button>
          {/each}
        </div>
        {#if mixer.sleepAction === 'sleep' || mixer.sleepAction === 'shutdown'}
          <p class="px-2.5 pt-2 text-[10.5px] leading-snug text-[var(--ca-text-muted)]">{t('sleep.powerNote')}</p>
        {/if}
      </div>
    {/if}
  </div>

  <!-- Master volume -->
  <div class="hidden sm:flex items-center gap-2 w-40">
    <button
      type="button"
      onclick={toggleMasterMute}
      class="text-[var(--ca-text-secondary)] hover:text-[var(--ca-text-primary)]"
      title={mixer.masterMuted ? t('master.unmute') : t('master.mute')}
      aria-label={mixer.masterMuted ? t('master.unmute') : t('master.mute')}
    >
      <Icon name={masterVal === 0 ? 'volume-x' : 'volume'} size={17} />
    </button>
    <input
      type="range"
      min="0"
      max="1"
      step="0.01"
      value={masterVal}
      oninput={(e) => setMaster(Number((e.target as HTMLInputElement).value))}
      class="calm-range"
      style="--val:{Math.round(masterVal * 100)}"
      aria-label={t('master.volume')}
    />
  </div>
</div>
