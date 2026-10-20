<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import { invoke } from '@tauri-apps/api/core';

  interface TimerSnapshot {
    phase: string;
    remaining: number;
    paused: boolean;
    sessions_today: number;
    focus_seconds_today: number;
  }

  interface Channel {
    id: string;
    name: string;
    icon: string;
    volume: number;
    muted: boolean;
    active: boolean;
  }

  interface Preset {
    id: string;
    name: string;
    channels: { id: string; volume: number; active: boolean }[];
  }

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
  let timerInterval: any = null;

  let channels = $state<Channel[]>([
    { id: 'ch-rain', name: 'Rain', icon: '🌧️', volume: 0.6, muted: false, active: true },
    { id: 'ch-waves', name: 'Ocean Waves', icon: '🌊', volume: 0.35, muted: false, active: true },
    { id: 'ch-birds', name: 'Forest Birds', icon: '🌲', volume: 0.5, muted: false, active: false },
    { id: 'ch-cafe', name: 'Coffee Shop', icon: '☕', volume: 0.4, muted: false, active: false },
    { id: 'ch-fire', name: 'Fireplace', icon: '🔥', volume: 0.6, muted: false, active: false },
    { id: 'ch-wind', name: 'Wind', icon: '🍃', volume: 0.3, muted: false, active: false },
    { id: 'ch-stream', name: 'Stream', icon: '🏞️', volume: 0.45, muted: false, active: false },
    { id: 'ch-noise', name: 'Brown Noise', icon: '⚪', volume: 0.25, muted: false, active: true },
  ]);

  const PRESETS: Preset[] = [
    {
      id: 'p-deep-focus',
      name: 'Deep Focus',
      channels: [
        { id: 'ch-rain', volume: 0.7, active: true },
        { id: 'ch-noise', volume: 0.4, active: true },
      ],
    },
    {
      id: 'p-rain-sleep',
      name: 'Rain Sleep',
      channels: [
        { id: 'ch-rain', volume: 0.8, active: true },
        { id: 'ch-waves', volume: 0.4, active: true },
        { id: 'ch-fire', volume: 0.3, active: true },
      ],
    },
    {
      id: 'p-cafe',
      name: 'Cafe Ambience',
      channels: [
        { id: 'ch-cafe', volume: 0.7, active: true },
        { id: 'ch-rain', volume: 0.3, active: true },
      ],
    },
    {
      id: 'p-wind-night',
      name: 'Wind Night',
      channels: [
        { id: 'ch-wind', volume: 0.6, active: true },
        { id: 'ch-fire', volume: 0.5, active: true },
      ],
    },
    {
      id: 'p-brown-noise',
      name: 'Pure Brown Noise',
      channels: [
        { id: 'ch-noise', volume: 0.8, active: true },
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
      // Keep local state
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

  function applyPreset(p: Preset) {
    selectedPreset = p.id;
    channels = channels.map((ch) => {
      const match = p.channels.find((pc) => pc.id === ch.id);
      if (match) {
        return { ...ch, volume: match.volume, active: match.active };
      }
      return { ...ch, active: false };
    });
  }

  function toggleSolo(chId: string) {
    const isCurrentlySolo = channels.filter((c) => c.active).length === 1 && channels.find((c) => c.id === chId)?.active;
    if (isCurrentlySolo) {
      // Un-solo: activate original mix
      channels = channels.map((c) => ({ ...c, active: true }));
    } else {
      channels = channels.map((c) => ({ ...c, active: c.id === chId }));
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
      isMasterMuted = !isMasterMuted;
    } else if ((e.ctrlKey || e.metaKey) && e.shiftKey && e.key === 'N') {
      e.preventDefault();
      skipPhase();
    }
  }

  onMount(() => {
    void syncTimer();
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
  });

  const progressPercent = $derived.by(() => {
    const total = timerState.phase === 'FOCUS' ? 25 * 60 : timerState.phase === 'LONG BREAK' ? 15 * 60 : 5 * 60;
    return Math.max(0, Math.min(100, Math.round(((total - timerState.remaining) / total) * 100)));
  });
</script>

<div class={`flex-1 flex flex-col h-full bg-[#0A0A0C] text-[#EDEDED] overflow-y-auto p-6 space-y-6 font-sans transition-all duration-300 ${isDistractionShield ? 'backdrop-blur-xl bg-black/95' : ''}`}>
  <!-- Top Bar -->
  <div class="flex items-center justify-between border-b border-[#272732] pb-4">
    <div class="flex items-center gap-3">
      <div class="w-8 h-8 rounded-lg bg-teal-500/10 border border-teal-500/30 flex items-center justify-center text-teal-400 font-bold text-sm">
        🌿
      </div>
      <div>
        <h1 class="text-base font-bold tracking-tight text-white flex items-center gap-2">
          CACalm
          <span class="text-[10px] font-mono px-2 py-0.5 rounded-full bg-teal-500/10 text-teal-400 border border-teal-500/20">v0.1.1</span>
        </h1>
        <p class="text-[11px] text-neutral-400">Offline multi-channel ambient sound mixer & Pomodoro focus engine</p>
      </div>
    </div>

    <div class="flex items-center gap-3">
      <!-- Distraction Shield Button -->
      <button
        onclick={() => (isDistractionShield = !isDistractionShield)}
        class={`px-3 py-1.5 text-xs font-semibold rounded-lg border transition-all cursor-pointer flex items-center gap-1.5 ${isDistractionShield ? 'bg-teal-500 text-black border-teal-400 shadow-[0_0_12px_rgba(20,184,166,0.5)]' : 'bg-[#18181F] text-neutral-300 border-[#272732] hover:bg-[#202028]'}`}
        title="Beralih ke mode Distraction Shield tanpa gangguan visual"
      >
        <span>🛡️</span>
        <span>{isDistractionShield ? 'Shield Active' : 'Distraction Shield'}</span>
      </button>

      <!-- Master Volume Control -->
      <div class="flex items-center gap-2 bg-[#18181F] px-3 py-1.5 rounded-lg border border-[#272732] text-xs">
        <button
          onclick={() => (isMasterMuted = !isMasterMuted)}
          class="text-neutral-400 hover:text-white"
          title="Mute Master (Ctrl+M)"
        >
          {isMasterMuted || masterVolume === 0 ? '🔇' : '🔊'}
        </button>
        <input
          type="range"
          min="0"
          max="1"
          step="0.01"
          bind:value={masterVolume}
          disabled={isMasterMuted}
          class="w-20 accent-teal-500 cursor-pointer"
        />
        <span class="font-mono text-[10px] text-teal-400 w-7 text-right">
          {isMasterMuted ? '0%' : `${Math.round(masterVolume * 100)}%`}
        </span>
      </div>
    </div>
  </div>

  <!-- Pomodoro Focus Timer Section -->
  <div class="p-6 rounded-2xl bg-[#111827]/70 border border-[#272732] flex flex-col md:flex-row items-center justify-between gap-6 shadow-xl relative overflow-hidden">
    <!-- Background progress glow -->
    <div class="absolute inset-0 bg-teal-500/5 pointer-events-none"></div>

    <div class="flex items-center gap-6 z-10">
      <!-- Circular Progress Ring (SVG) -->
      <div class="relative w-28 h-28 flex items-center justify-center shrink-0">
        <svg class="w-full h-full -rotate-90" viewBox="0 0 100 100">
          <circle cx="50" cy="50" r="42" fill="none" stroke="#1f2937" stroke-width="8" />
          <circle
            cx="50"
            cy="50"
            r="42"
            fill="none"
            stroke="#14b8a6"
            stroke-width="8"
            stroke-linecap="round"
            stroke-dasharray="264"
            stroke-dashoffset={264 - (264 * progressPercent) / 100}
            class="transition-all duration-500"
          />
        </svg>
        <div class="absolute flex flex-col items-center justify-center text-center">
          <span class="text-xl font-bold font-mono text-white tracking-tight">{formatTime(timerState.remaining)}</span>
          <span class="text-[9px] font-mono text-teal-400 uppercase tracking-widest">{timerState.phase}</span>
        </div>
      </div>

      <!-- Timer Info & Stats -->
      <div class="space-y-1 z-10">
        <div class="flex items-center gap-2">
          <span class={`text-[10px] font-bold px-2 py-0.5 rounded-full uppercase tracking-wider ${timerState.phase === 'FOCUS' ? 'bg-teal-500/10 text-teal-400 border border-teal-500/30' : 'bg-amber-500/10 text-amber-400 border border-amber-500/30'}`}>
            {timerState.phase}
          </span>
          {#if timerState.paused}
            <span class="text-[10px] font-mono text-neutral-400">PAUSED</span>
          {:else}
            <span class="text-[10px] font-mono text-emerald-400 flex items-center gap-1">
              <span class="w-1.5 h-1.5 rounded-full bg-emerald-400 animate-pulse"></span> RUNNING
            </span>
          {/if}
        </div>
        <h2 class="text-lg font-bold text-white">
          {timerState.phase === 'FOCUS' ? 'Fokus Penuh & Tanpa Distraksi' : 'Waktu Istirahat Sejenak'}
        </h2>
        <p class="text-xs text-neutral-400 font-mono">
          Sesi Selesai Hari Ini: <span class="text-white font-bold">{timerState.sessions_today}</span> · Fokus: <span class="text-white font-bold">{Math.floor(timerState.focus_seconds_today / 60)} min</span>
        </p>
      </div>
    </div>

    <!-- Timer Control Buttons -->
    <div class="flex items-center gap-3 z-10">
      <button
        onclick={togglePlayPause}
        class={`px-5 py-2.5 rounded-xl font-bold text-xs shadow-md transition-all cursor-pointer flex items-center gap-2 ${timerState.paused ? 'bg-teal-500 hover:bg-teal-400 text-black' : 'bg-neutral-800 hover:bg-neutral-700 text-white border border-[#3f3f46]'}`}
      >
        <span>{timerState.paused ? '▶' : '⏸'}</span>
        <span>{timerState.paused ? 'Mulai Sesi (Space)' : 'Jeda (Space)'}</span>
      </button>

      <button
        onclick={resetTimer}
        class="p-2.5 rounded-xl bg-[#18181F] hover:bg-[#202028] border border-[#272732] text-neutral-300 hover:text-white transition-colors cursor-pointer text-xs"
        title="Reset Timer (Ctrl+R)"
      >
        ↺ Reset
      </button>

      <button
        onclick={skipPhase}
        class="p-2.5 rounded-xl bg-[#18181F] hover:bg-[#202028] border border-[#272732] text-neutral-300 hover:text-white transition-colors cursor-pointer text-xs"
        title="Lompat ke Fase Berikutnya (Ctrl+Shift+N)"
      >
        ⏭ Lewati
      </button>
    </div>
  </div>

  <!-- Presets Selection Bar -->
  <div class="flex items-center justify-between gap-2 overflow-x-auto pb-1">
    <div class="flex items-center gap-2 text-xs font-semibold">
      <span class="text-neutral-500 uppercase tracking-wider text-[10px] mr-1">Presets:</span>
      {#each PRESETS as preset}
        <button
          onclick={() => applyPreset(preset)}
          class={`px-3 py-1.5 rounded-xl border text-xs font-medium transition-all cursor-pointer ${selectedPreset === preset.id ? 'bg-teal-500/10 text-teal-300 border-teal-500/40 shadow-xs' : 'bg-[#121217] text-neutral-400 border-[#272732] hover:text-white hover:bg-[#18181F]'}`}
        >
          {preset.name}
        </button>
      {/each}
    </div>
  </div>

  <!-- Channel Mixer Section (8 Channels) -->
  <div class="space-y-3">
    <div class="flex items-center justify-between">
      <span class="text-xs font-bold uppercase tracking-wider text-neutral-400">
        Multi-Channel Acoustic Mixer (8 Channels)
      </span>
      <span class="text-[10px] font-mono text-neutral-500">
        Gapless loop · 0 akun · 100% offline
      </span>
    </div>

    <div class="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-4 gap-4">
      {#each channels as channel (channel.id)}
        <div
          class={`p-4 rounded-xl border transition-all flex flex-col justify-between space-y-3 ${channel.active ? 'bg-[#111827] border-teal-500/40 shadow-sm' : 'bg-[#0f0f13] border-[#272732] opacity-60'}`}
        >
          <!-- Channel Header -->
          <div class="flex items-center justify-between">
            <div class="flex items-center gap-2.5">
              <span class="text-lg">{channel.icon}</span>
              <div>
                <h3 class="font-bold text-xs text-white">{channel.name}</h3>
                <span class="text-[9px] font-mono text-neutral-500">
                  {channel.active ? (channel.muted ? 'Muted' : 'Playing') : 'Inactive'}
                </span>
              </div>
            </div>

            <button
              onclick={() => (channel.active = !channel.active)}
              class={`w-6 h-6 rounded-lg text-xs font-bold flex items-center justify-center transition-colors cursor-pointer ${channel.active ? 'bg-teal-500 text-black' : 'bg-[#18181F] text-neutral-500 border border-[#272732]'}`}
              title={channel.active ? 'Matikan Suara' : 'Hidupkan Suara'}
            >
              {channel.active ? '✓' : '+'}
            </button>
          </div>

          <!-- Volume Slider -->
          <div class="space-y-1">
            <div class="flex justify-between items-center text-[10px] font-mono">
              <span class="text-neutral-400">Gain</span>
              <span class="text-teal-400 font-bold">{channel.muted ? '0%' : `${Math.round(channel.volume * 100)}%`}</span>
            </div>
            <input
              type="range"
              min="0"
              max="1"
              step="0.01"
              bind:value={channel.volume}
              disabled={!channel.active || channel.muted}
              class="w-full accent-teal-500 cursor-pointer disabled:opacity-40"
            />
          </div>

          <!-- Solo & Mute Buttons -->
          <div class="flex items-center gap-2 pt-1 border-t border-[#1f2937] text-[10px] font-mono">
            <button
              onclick={() => (channel.muted = !channel.muted)}
              disabled={!channel.active}
              class={`flex-1 py-1 rounded border text-center transition-colors cursor-pointer disabled:opacity-30 ${channel.muted ? 'bg-amber-500/20 text-amber-300 border-amber-500/40' : 'bg-[#18181F] text-neutral-400 border-[#272732] hover:text-white'}`}
            >
              {channel.muted ? 'UNMUTE' : 'MUTE'}
            </button>

            <button
              onclick={() => toggleSolo(channel.id)}
              disabled={!channel.active}
              class="flex-1 py-1 rounded bg-[#18181F] hover:bg-[#202028] text-neutral-400 hover:text-white border border-[#272732] text-center transition-colors cursor-pointer disabled:opacity-30"
            >
              SOLO
            </button>
          </div>
        </div>
      {/each}
    </div>
  </div>
</div>
