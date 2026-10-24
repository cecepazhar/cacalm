<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import { invoke } from '@tauri-apps/api/core';
  import { ProGate, Badge, Button, Slider } from '@cecepazhar/caui';
  import { t } from '$lib/i18n/index.svelte';
  import { getPro, openUpgradeModal } from '$lib/stores/pro.svelte';
  import { webAudio } from '$lib/audio/webAudio';

  interface TimerSnapshot {
    phase: string;
    remaining: number;
    paused: boolean;
    sessions_today: number;
    focus_seconds_today: number;
  }

  interface Channel {
    id: string;
    nameKey: string;
    descKey: string;
    icon: string;
    volume: number;
    pan: number;
    muted: boolean;
    active: boolean;
    isPro: boolean;
  }

  interface Preset {
    id: string;
    nameKey: string;
    channels: { id: string; volume: number; active: boolean; pan?: number }[];
  }

  const pro = getPro();

  let timerState = $state<TimerSnapshot>({
    phase: 'FOCUS',
    remaining: 25 * 60,
    paused: true,
    sessions_today: 0,
    focus_seconds_today: 0,
  });

  let masterVolume = $state(0.8);
  let isMasterMuted = $state(false);
  let isDistractionShield = $state(false);
  let timerInterval: ReturnType<typeof setInterval> | null = null;

  let channels = $state<Channel[]>([
    { id: 'ch-rain', nameKey: 'rain', descKey: 'rainDesc', icon: '🌧️', volume: 0.6, pan: 0.0, muted: false, active: true, isPro: false },
    { id: 'ch-waves', nameKey: 'waves', descKey: 'wavesDesc', icon: '🌊', volume: 0.35, pan: -0.2, muted: false, active: true, isPro: false },
    { id: 'ch-wind', nameKey: 'wind', descKey: 'windDesc', icon: '🍃', volume: 0.3, pan: 0.3, muted: false, active: false, isPro: false },
    { id: 'ch-noise', nameKey: 'noise', descKey: 'noiseDesc', icon: '⚪', volume: 0.25, pan: 0.0, muted: false, active: true, isPro: false },
    { id: 'ch-birds', nameKey: 'birds', descKey: 'birdsDesc', icon: '🌲', volume: 0.5, pan: -0.3, muted: false, active: false, isPro: true },
    { id: 'ch-cafe', nameKey: 'cafe', descKey: 'cafeDesc', icon: '☕', volume: 0.4, pan: 0.2, muted: false, active: false, isPro: true },
    { id: 'ch-fire', nameKey: 'fire', descKey: 'fireDesc', icon: '🔥', volume: 0.6, pan: 0.1, muted: false, active: false, isPro: true },
    { id: 'ch-stream', nameKey: 'stream', descKey: 'streamDesc', icon: '🏞️', volume: 0.45, pan: -0.1, muted: false, active: false, isPro: true },
    { id: 'ch-binaural', nameKey: 'binaural', descKey: 'binauralDesc', icon: '🧠', volume: 0.3, pan: 0.0, muted: false, active: false, isPro: true },
    { id: 'ch-spatial', nameKey: 'spatial', descKey: 'spatialDesc', icon: '🎧', volume: 0.5, pan: 0.0, muted: false, active: false, isPro: true },
  ]);

  const PRESETS: Preset[] = [
    {
      id: 'p-deep-focus',
      nameKey: 'deepFocus',
      channels: [
        { id: 'ch-rain', volume: 0.7, active: true, pan: 0.0 },
        { id: 'ch-noise', volume: 0.4, active: true, pan: 0.0 },
      ],
    },
    {
      id: 'p-rain-sleep',
      nameKey: 'rainSleep',
      channels: [
        { id: 'ch-rain', volume: 0.8, active: true, pan: -0.2 },
        { id: 'ch-waves', volume: 0.4, active: true, pan: 0.2 },
        { id: 'ch-fire', volume: 0.3, active: true, pan: 0.0 },
      ],
    },
    {
      id: 'p-cafe',
      nameKey: 'cafeAmbience',
      channels: [
        { id: 'ch-cafe', volume: 0.7, active: true, pan: 0.2 },
        { id: 'ch-rain', volume: 0.3, active: true, pan: -0.2 },
      ],
    },
    {
      id: 'p-wind-night',
      nameKey: 'windNight',
      channels: [
        { id: 'ch-wind', volume: 0.6, active: true, pan: 0.3 },
        { id: 'ch-fire', volume: 0.5, active: true, pan: -0.1 },
      ],
    },
    {
      id: 'p-brown-noise',
      nameKey: 'brownNoise',
      channels: [
        { id: 'ch-noise', volume: 0.8, active: true, pan: 0.0 },
      ],
    },
  ];

  let selectedPreset = $state<string>('p-deep-focus');

  function formatTime(secs: number): string {
    const m = Math.floor(secs / 60);
    const s = secs % 60;
    return `${m}:${s < 10 ? '0' : ''}${s}`;
  }

  async function syncTimer() {
    try {
      timerState = await invoke<TimerSnapshot>('get_timer_snapshot');
    } catch {
      // Local state fallback
    }
  }

  async function togglePlayPause() {
    try {
      if (timerState.paused) {
        timerState = await invoke<TimerSnapshot>('timer_resume');
      } else {
        timerState = await invoke<TimerSnapshot>('timer_pause');
      }
    } catch {
      timerState.paused = !timerState.paused;
    }
  }

  async function resetTimer() {
    try {
      timerState = await invoke<TimerSnapshot>('timer_reset');
    } catch {
      timerState.remaining = 25 * 60;
      timerState.paused = true;
    }
  }

  async function skipPhase() {
    try {
      timerState = await invoke<TimerSnapshot>('timer_skip');
    } catch {
      timerState.phase = timerState.phase === 'FOCUS' ? 'SHORT BREAK' : 'FOCUS';
      timerState.remaining = timerState.phase === 'FOCUS' ? 25 * 60 : 5 * 60;
    }
  }

  // --- Audio IPC Helpers ---

  async function updateMasterVolume(val: number) {
    masterVolume = val;
    const effective = isMasterMuted ? 0 : masterVolume;
    webAudio.setMasterVolume(effective);
    try {
      await invoke('set_master_volume', { volume: effective });
    } catch {
      // Ignore in browser
    }
  }

  async function toggleMasterMute() {
    isMasterMuted = !isMasterMuted;
    const effective = isMasterMuted ? 0 : masterVolume;
    webAudio.setMasterVolume(effective);
    try {
      await invoke('set_master_volume', { volume: effective });
    } catch {
      // Ignore in browser
    }
  }

  async function toggleChannelActive(ch: Channel) {
    if (ch.isPro && !pro.isPro) {
      openUpgradeModal();
      return;
    }

    ch.active = !ch.active;
    if (ch.active) {
      webAudio.playChannel(ch.id, ch.muted ? 0 : ch.volume, ch.pan);
      try {
        await invoke('play_channel', { channelId: ch.id, volume: ch.muted ? 0 : ch.volume });
        await invoke('set_channel_pan', { channelId: ch.id, pan: ch.pan });
      } catch {
        // Ignore in browser
      }
    } else {
      webAudio.stopChannel(ch.id);
      try {
        await invoke('stop_channel', { channelId: ch.id });
      } catch {
        // Ignore in browser
      }
    }
  }

  async function handleVolumeChange(ch: Channel, val: number) {
    ch.volume = val;
    const vol = ch.muted ? 0 : ch.volume;
    webAudio.setChannelVolume(ch.id, vol);
    try {
      await invoke('set_channel_volume', { channelId: ch.id, volume: vol });
    } catch {
      // Ignore in browser
    }
  }

  async function handlePanChange(ch: Channel, val: number) {
    ch.pan = val;
    webAudio.setChannelPan(ch.id, ch.pan);
    try {
      await invoke('set_channel_pan', { channelId: ch.id, pan: ch.pan });
    } catch {
      // Ignore in browser
    }
  }

  async function toggleChannelMute(ch: Channel) {
    ch.muted = !ch.muted;
    const vol = ch.muted ? 0 : ch.volume;
    webAudio.setChannelVolume(ch.id, vol);
    try {
      await invoke('set_channel_volume', { channelId: ch.id, volume: vol });
    } catch {
      // Ignore in browser
    }
  }

  async function toggleSolo(chId: string) {
    const isCurrentlySolo = channels.filter((c) => c.active).length === 1 && channels.find((c) => c.id === chId)?.active;
    for (const c of channels) {
      if (c.isPro && !pro.isPro) continue;
      if (isCurrentlySolo) {
        c.active = true;
        webAudio.playChannel(c.id, c.muted ? 0 : c.volume, c.pan);
        try {
          await invoke('play_channel', { channelId: c.id, volume: c.muted ? 0 : c.volume });
        } catch {}
      } else {
        c.active = c.id === chId;
        if (c.active) {
          webAudio.playChannel(c.id, c.muted ? 0 : c.volume, c.pan);
          try {
            await invoke('play_channel', { channelId: c.id, volume: c.muted ? 0 : c.volume });
          } catch {}
        } else {
          webAudio.stopChannel(c.id);
          try {
            await invoke('stop_channel', { channelId: c.id });
          } catch {}
        }
      }
    }
  }

  async function applyPreset(p: Preset) {
    selectedPreset = p.id;
    for (const ch of channels) {
      const match = p.channels.find((pc) => pc.id === ch.id);
      if (match) {
        if (ch.isPro && !pro.isPro) {
          ch.active = false;
          webAudio.stopChannel(ch.id);
          try {
            await invoke('stop_channel', { channelId: ch.id });
          } catch {}
          continue;
        }
        ch.volume = match.volume;
        ch.active = match.active;
        if (match.pan !== undefined) ch.pan = match.pan;
        webAudio.playChannel(ch.id, ch.muted ? 0 : ch.volume, ch.pan);
        try {
          await invoke('play_channel', { channelId: ch.id, volume: ch.muted ? 0 : ch.volume });
          await invoke('set_channel_pan', { channelId: ch.id, pan: ch.pan });
        } catch {}
      } else {
        ch.active = false;
        webAudio.stopChannel(ch.id);
        try {
          await invoke('stop_channel', { channelId: ch.id });
        } catch {}
      }
    }
  }

  function handleKeydown(e: KeyboardEvent) {
    if (e.code === 'Space' && (e.target as HTMLElement)?.tagName !== 'INPUT') {
      e.preventDefault();
      togglePlayPause();
    } else if ((e.ctrlKey || e.metaKey) && e.key === 'r') {
      e.preventDefault();
      resetTimer();
    } else if ((e.ctrlKey || e.metaKey) && e.key === 'm') {
      e.preventDefault();
      toggleMasterMute();
    } else if ((e.ctrlKey || e.metaKey) && e.shiftKey && e.key === 'N') {
      e.preventDefault();
      skipPhase();
    }
  }

  onMount(() => {
    void syncTimer();
    // Initialize active channels
    for (const ch of channels) {
      if (ch.active) {
        if (ch.isPro && !pro.isPro) {
          ch.active = false;
        } else {
          webAudio.playChannel(ch.id, ch.muted ? 0 : ch.volume, ch.pan);
          try {
            invoke('play_channel', { channelId: ch.id, volume: ch.muted ? 0 : ch.volume });
            invoke('set_channel_pan', { channelId: ch.id, pan: ch.pan });
          } catch {}
        }
      }
    }

    timerInterval = setInterval(async () => {
      if (!timerState.paused) {
        try {
          timerState = await invoke<TimerSnapshot>('timer_tick', { elapsedSecs: 1 });
        } catch {
          if (timerState.remaining > 0) {
            timerState.remaining--;
          } else {
            skipPhase();
          }
        }
      }
    }, 1000);

    window.addEventListener('keydown', handleKeydown);
  });

  onDestroy(() => {
    if (timerInterval) clearInterval(timerInterval);
    if (typeof window !== 'undefined') {
      window.removeEventListener('keydown', handleKeydown);
    }
    for (const ch of channels) {
      webAudio.stopChannel(ch.id);
    }
  });

  const progressPercent = $derived.by(() => {
    const total = timerState.phase === 'FOCUS' ? 25 * 60 : timerState.phase === 'LONG BREAK' ? 15 * 60 : 5 * 60;
    return Math.max(0, Math.min(100, Math.round(((total - timerState.remaining) / total) * 100)));
  });

  const activeChannelCount = $derived(channels.filter((c) => c.active).length);
</script>

<div class={`flex-1 min-h-full w-full bg-[var(--ca-canvas)] text-[var(--ca-text-primary)] overflow-y-auto p-4 md:p-6 space-y-6 transition-all duration-300 ${isDistractionShield ? 'backdrop-blur-xl bg-black/95' : ''}`}>
  <!-- Single Page Action Bar (Double header removed!) -->
  <div class="flex flex-col sm:flex-row sm:items-center justify-between gap-4 border-b border-[var(--ca-border)] pb-4">
    <div>
      <h1 class="text-base font-bold tracking-tight text-[var(--ca-text-primary)] flex items-center gap-2">
        {t('mixer.title')}
        <Badge variant="brand" size="xs">
          {t('mixer.activeChannels', { count: activeChannelCount })}
        </Badge>
      </h1>
      <p class="text-xs text-[var(--ca-text-muted)] mt-0.5">
        {t('mixer.subtitle')}
      </p>
    </div>

    <div class="flex items-center gap-3">
      <!-- Distraction Shield Button -->
      <button
        type="button"
        onclick={() => (isDistractionShield = !isDistractionShield)}
        class={`px-3 py-1.5 text-xs font-semibold rounded-lg border transition-all cursor-pointer flex items-center gap-1.5 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-[var(--ca-brand)] ${
          isDistractionShield
            ? 'bg-[var(--ca-brand)] text-white border-[var(--ca-brand)] shadow-sm'
            : 'bg-[var(--ca-surface)] text-[var(--ca-text-secondary)] border-[var(--ca-border)] hover:bg-[var(--ca-surface-elevated)] hover:text-[var(--ca-text-primary)]'
        }`}
        title={t('timer.distractionShieldTooltip')}
      >
        <span>🛡️</span>
        <span>{isDistractionShield ? t('timer.distractionShieldActive') : t('timer.distractionShield')}</span>
      </button>

      <!-- Master Volume Control -->
      <div class="flex items-center gap-2 bg-[var(--ca-surface)] px-3 py-1.5 rounded-lg border border-[var(--ca-border)] text-xs shadow-xs">
        <button
          type="button"
          onclick={toggleMasterMute}
          class="text-[var(--ca-text-muted)] hover:text-[var(--ca-text-primary)] transition-colors cursor-pointer text-sm"
          title={isMasterMuted ? t('mixer.unmute') : t('mixer.mute')}
        >
          {isMasterMuted || masterVolume === 0 ? '🔇' : '🔊'}
        </button>
        <input
          type="range"
          min="0"
          max="1"
          step="0.01"
          value={masterVolume}
          oninput={(e) => updateMasterVolume(Number((e.target as HTMLInputElement).value))}
          disabled={isMasterMuted}
          aria-label={t('mixer.masterVolume')}
          class="w-20 accent-[var(--ca-brand)] cursor-pointer disabled:opacity-40"
        />
        <span class="font-mono text-[10px] text-[var(--ca-brand)] font-bold w-7 text-right">
          {isMasterMuted ? '0%' : `${Math.round(masterVolume * 100)}%`}
        </span>
      </div>
    </div>
  </div>

  <!-- Pomodoro Focus Timer Section -->
  <div class="p-4 md:p-6 rounded-2xl bg-[var(--ca-surface)] border border-[var(--ca-border)] flex flex-col md:flex-row items-center justify-between gap-4 md:gap-6 shadow-sm relative transition-colors shrink-0">
    <div class="flex items-center gap-4 md:gap-6 z-10 shrink-0">
      <!-- Circular Progress Ring (SVG) -->
      <div class="relative w-24 md:w-28 h-24 md:h-28 flex items-center justify-center shrink-0">
        <svg class="w-full h-full -rotate-90 overflow-visible" viewBox="0 0 100 100">
          <circle cx="50" cy="50" r="42" fill="none" stroke="currentColor" class="text-[var(--ca-surface-subtle)]" stroke-width="8" />
          <circle
            cx="50"
            cy="50"
            r="42"
            fill="none"
            stroke="var(--ca-brand)"
            stroke-width="8"
            stroke-linecap="round"
            stroke-dasharray="264"
            stroke-dashoffset={264 - (264 * progressPercent) / 100}
            class="transition-all duration-500"
          />
        </svg>
        <div class="absolute flex flex-col items-center justify-center text-center w-full h-full px-1 py-0.5">
          <span class="text-lg md:text-xl font-bold font-mono text-[var(--ca-text-primary)] tracking-tight">
            {formatTime(timerState.remaining)}
          </span>
          <span class="text-[8px] md:text-[9px] font-mono text-[var(--ca-brand)] uppercase tracking-widest font-semibold">
            {timerState.phase}
          </span>
        </div>
      </div>

      <!-- Timer Info & Stats -->
      <div class="space-y-1.5 z-10 flex-shrink min-w-0">
        <div class="flex items-center gap-2 flex-wrap">
          <Badge variant={timerState.phase === 'FOCUS' ? 'brand' : 'warning'} size="xs">
            {timerState.phase}
          </Badge>
          {#if timerState.paused}
            <span class="text-[10px] font-mono text-[var(--ca-text-muted)] font-semibold">
              {t('timer.pause').toUpperCase()}
            </span>
          {:else}
            <span class="text-[10px] font-mono text-[var(--ca-brand)] flex items-center gap-1 font-semibold">
              <span class="w-1.5 h-1.5 rounded-full bg-[var(--ca-brand)] animate-pulse"></span>
              {t('timer.resume').toUpperCase()}
            </span>
          {/if}
        </div>
        <h2 class="text-base font-bold text-[var(--ca-text-primary)]">
          {timerState.phase === 'FOCUS' ? t('timer.focus') : t('timer.shortBreak')}
        </h2>
        <p class="text-xs text-[var(--ca-text-secondary)] font-mono">
          {t('timer.sessionsToday')}: <strong class="text-[var(--ca-text-primary)]">{timerState.sessions_today}</strong> · {t('timer.focusTimeToday')}: <strong class="text-[var(--ca-text-primary)]">{Math.floor(timerState.focus_seconds_today / 60)} {t('timer.minutes')}</strong>
        </p>
      </div>
    </div>

    <!-- Timer Control Buttons -->
    <div class="flex items-center gap-2.5 z-10 flex-wrap">
      <Button
        variant={timerState.paused ? 'brand' : 'secondary'}
        size="md"
        onclick={togglePlayPause}
        class="font-bold gap-1.5"
      >
        <span>{timerState.paused ? '▶' : '⏸'}</span>
        <span>{timerState.paused ? `${t('timer.start')} (Space)` : `${t('timer.pause')} (Space)`}</span>
      </Button>

      <Button
        variant="outline"
        size="md"
        onclick={resetTimer}
        title="Reset Timer (Ctrl+R)"
      >
        ↺ {t('timer.reset')}
      </Button>

      <Button
        variant="outline"
        size="md"
        onclick={skipPhase}
        title="Skip Phase (Shift+N)"
      >
        ⏭ {t('timer.skip')}
      </Button>
    </div>
  </div>

  <!-- Presets Selection Bar -->
  <div class="space-y-1.5">
    <div class="flex items-center gap-2 overflow-x-auto pb-1">
      <span class="text-[var(--ca-text-muted)] uppercase tracking-wider text-[10px] font-bold mr-1">
        {t('presets.title')}:
      </span>
      {#each PRESETS as preset (preset.id)}
        <button
          type="button"
          onclick={() => applyPreset(preset)}
          class={`px-3 py-1.5 rounded-xl border text-xs font-medium transition-all cursor-pointer focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-[var(--ca-brand)] ${
            selectedPreset === preset.id
              ? 'bg-[var(--ca-brand)]/15 text-[var(--ca-brand)] border-[var(--ca-brand)]/40 shadow-xs font-semibold'
              : 'bg-[var(--ca-surface)] text-[var(--ca-text-secondary)] border-[var(--ca-border)] hover:text-[var(--ca-text-primary)] hover:bg-[var(--ca-surface-elevated)]'
          }`}
        >
          {t(`presets.${preset.nameKey}`)}
        </button>
      {/each}
    </div>
  </div>

  <!-- Channel Mixer Section -->
  <div class="space-y-3">
    <div class="flex items-center justify-between">
      <h3 class="text-xs font-bold uppercase tracking-wider text-[var(--ca-text-secondary)]">
        {t('mixer.soundCount', { count: channels.length })}
      </h3>
      <span class="text-[10px] font-mono text-[var(--ca-text-muted)]">
        {t('timer.keyboardShortcuts')}
      </span>
    </div>

    <!-- 10 Channels Grid (Free + Pro Exclusive) -->
    <div class="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4 gap-4">
      {#each channels as channel (channel.id)}
        {#if channel.isPro && !pro.isPro}
          <!-- Locked Pro Channel using CAUI ProGate -->
          <ProGate
            featureName={t(`channels.${channel.nameKey}`)}
            locked={true}
            onUpgrade={openUpgradeModal}
            class="flex flex-col justify-center min-h-[160px]"
          />
        {:else}
          <!-- Interactive Channel Card (Free or Unlocked Pro) -->
          <div
            class={`p-4 rounded-xl border transition-all flex flex-col justify-between space-y-3 ${
              channel.active
                ? 'bg-[var(--ca-surface)] border-[var(--ca-brand)]/40 shadow-xs'
                : 'bg-[var(--ca-surface-subtle)]/40 border-[var(--ca-border)] opacity-70 hover:opacity-100'
            }`}
          >
            <!-- Channel Header -->
            <div class="flex items-center justify-between">
              <div class="flex items-center gap-2.5">
                <span class="text-lg">{channel.icon}</span>
                <div>
                  <div class="flex items-center gap-1.5">
                    <h4 class="font-bold text-xs text-[var(--ca-text-primary)]">
                      {t(`channels.${channel.nameKey}`)}
                    </h4>
                    {#if channel.isPro}
                      <Badge variant="brand" size="xs">
                        <span class="text-[9px] font-bold">PRO</span>
                      </Badge>
                    {/if}
                  </div>
                  <span class="text-[9px] font-mono text-[var(--ca-text-muted)]">
                    {channel.active ? (channel.muted ? t('mixer.mute') : t('mixer.active')) : t('mixer.inactive')}
                  </span>
                </div>
              </div>

              <!-- Power / Toggle Button -->
              <button
                type="button"
                onclick={() => toggleChannelActive(channel)}
                class={`w-6 h-6 rounded-lg text-xs font-bold flex items-center justify-center transition-all cursor-pointer focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-[var(--ca-brand)] ${
                  channel.active
                    ? 'bg-[var(--ca-brand)] text-white shadow-xs'
                    : 'bg-[var(--ca-surface-elevated)] text-[var(--ca-text-muted)] border border-[var(--ca-border)] hover:text-[var(--ca-text-primary)]'
                }`}
                title={channel.active ? t('mixer.inactive') : t('mixer.active')}
                aria-label={t(`channels.${channel.nameKey}`)}
              >
                {channel.active ? '✓' : '+'}
              </button>
            </div>

            <!-- Volume Slider -->
            <div class="space-y-1">
              <div class="flex justify-between items-center text-[10px] font-mono">
                <span class="text-[var(--ca-text-muted)]">{t('mixer.volume')}</span>
                <span class="text-[var(--ca-brand)] font-bold">
                  {channel.muted ? '0%' : `${Math.round(channel.volume * 100)}%`}
                </span>
              </div>
              <input
                type="range"
                min="0"
                max="1"
                step="0.01"
                value={channel.volume}
                oninput={(e) => handleVolumeChange(channel, Number((e.target as HTMLInputElement).value))}
                disabled={!channel.active || channel.muted}
                aria-label={`${t(`channels.${channel.nameKey}`)} ${t('mixer.volume')}`}
                class="w-full accent-[var(--ca-brand)] cursor-pointer disabled:opacity-40"
              />
            </div>

            <!-- Spatial Pan Slider -->
            <div class="space-y-1 pt-1 border-t border-[var(--ca-border)]/50">
              <div class="flex justify-between items-center text-[9px] font-mono">
                <span class="text-[var(--ca-text-muted)]">{t('mixer.pan')}</span>
                <span class="text-[var(--ca-text-secondary)]">
                  {channel.pan < -0.1 ? `${t('mixer.panLeft')} ${Math.abs(Math.round(channel.pan * 100))}%` : channel.pan > 0.1 ? `${t('mixer.panRight')} ${Math.round(channel.pan * 100)}%` : t('mixer.panCenter')}
                </span>
              </div>
              <input
                type="range"
                min="-1"
                max="1"
                step="0.05"
                value={channel.pan}
                oninput={(e) => handlePanChange(channel, Number((e.target as HTMLInputElement).value))}
                disabled={!channel.active}
                aria-label={`${t(`channels.${channel.nameKey}`)} ${t('mixer.pan')}`}
                class="w-full accent-[var(--ca-brand)] cursor-pointer disabled:opacity-40 h-1"
              />
            </div>

            <!-- Solo & Mute Buttons -->
            <div class="flex items-center gap-2 pt-1 border-t border-[var(--ca-border)]/50 text-[10px] font-mono">
              <button
                type="button"
                onclick={() => toggleChannelMute(channel)}
                disabled={!channel.active}
                class={`flex-1 py-1 rounded border text-center transition-colors cursor-pointer disabled:opacity-30 ${
                  channel.muted
                    ? 'bg-amber-500/20 text-amber-500 border-amber-500/40 font-bold'
                    : 'bg-[var(--ca-surface-elevated)] text-[var(--ca-text-secondary)] border-[var(--ca-border)] hover:text-[var(--ca-text-primary)]'
                }`}
              >
                {channel.muted ? t('mixer.unmute').toUpperCase() : t('mixer.mute').toUpperCase()}
              </button>

              <button
                type="button"
                onclick={() => toggleSolo(channel.id)}
                disabled={!channel.active}
                class="flex-1 py-1 rounded bg-[var(--ca-surface-elevated)] hover:bg-[var(--ca-surface-subtle)] text-[var(--ca-text-secondary)] hover:text-[var(--ca-text-primary)] border border-[var(--ca-border)] text-center transition-colors cursor-pointer disabled:opacity-30"
              >
                {t('mixer.solo').toUpperCase()}
              </button>
            </div>
          </div>
        {/if}
      {/each}
    </div>
  </div>
</div>
