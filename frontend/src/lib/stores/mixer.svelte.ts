// Global mixer state. Lives outside any page so sound keeps playing while the
// user navigates between Sounds / Focus / Library / Settings.

import { engine } from '$lib/audio/engine';
import { SOUND_BY_ID, type Scene } from '$lib/audio/catalog';
import { isTauri } from '$lib/env';

export interface ChannelState {
  volume: number;
  pan: number;
  muted: boolean;
}

export interface CustomSound {
  id: string; // "custom:<uuid>"
  name: string;
}

export interface UserMix {
  id: string;
  name: string;
  mix: { id: string; volume: number; pan: number }[];
}

const STORAGE_KEY = 'cacalm_mix_v2';
const SLEEP_FADE_SECS = 30;

const state = $state({
  selected: [] as string[],
  ch: {} as Record<string, ChannelState>,
  playing: false,
  master: 0.8,
  masterMuted: false,
  solo: null as string | null,
  loading: {} as Record<string, boolean>,
  sceneId: null as string | null,
  sleepMinutes: 0,
  sleepEndsAt: null as number | null,
  sleepFading: false,
  sleepAction: 'pause' as SleepAction,
  powerCountdown: null as number | null,
  powerCountdownAction: 'sleep' as 'sleep' | 'shutdown',
  now: Date.now(),
  custom: [] as CustomSound[],
  userMixes: [] as UserMix[],
});

export function getMixer() {
  return state;
}

function defaults(id: string): ChannelState {
  return { volume: SOUND_BY_ID.get(id)?.volume ?? 0.5, pan: 0, muted: false };
}

function chOf(id: string): ChannelState {
  if (!state.ch[id]) state.ch[id] = defaults(id);
  return state.ch[id];
}

function effective(id: string): number {
  const c = chOf(id);
  if (c.muted) return 0;
  if (state.solo && state.solo !== id) return 0;
  return c.volume;
}

function persist() {
  try {
    localStorage.setItem(
      STORAGE_KEY,
      JSON.stringify({
        selected: state.selected,
        ch: state.ch,
        master: state.master,
        sceneId: state.sceneId,
        userMixes: state.userMixes,
      })
    );
  } catch {
    /* storage unavailable */
  }
}

function restore() {
  try {
    const raw = localStorage.getItem(STORAGE_KEY);
    if (!raw) {
      // First launch: a gentle default mix, not playing until the user presses play.
      state.selected = ['ch-rain', 'ch-noise'];
      state.ch['ch-rain'] = { volume: 0.55, pan: 0, muted: false };
      state.ch['ch-noise'] = { volume: 0.3, pan: 0, muted: false };
      return;
    }
    const d = JSON.parse(raw);
    state.ch = d.ch ?? {};
    state.master = typeof d.master === 'number' ? d.master : 0.8;
    state.sceneId = d.sceneId ?? null;
    state.userMixes = Array.isArray(d.userMixes) ? d.userMixes : [];
    state.selected = (Array.isArray(d.selected) ? d.selected : []).filter(
      (id: string) => SOUND_BY_ID.has(id) || id.startsWith('custom:')
    );
  } catch {
    /* corrupted storage — start fresh */
  }
}

function startVoice(id: string) {
  const c = chOf(id);
  void engine.play(id, effective(id), c.pan).catch((e) => console.warn('play failed', id, e));
}

function syncVolumes() {
  for (const id of state.selected) engine.setVolume(id, effective(id));
}

// ---------------------------------------------------------------------------
// Public actions
// ---------------------------------------------------------------------------

export function isSelected(id: string): boolean {
  return state.selected.includes(id);
}

export function toggleSound(id: string) {
  state.sceneId = null;
  if (isSelected(id)) {
    state.selected = state.selected.filter((x) => x !== id);
    if (state.solo === id) {
      state.solo = null;
      syncVolumes();
    }
    engine.stop(id);
  } else {
    state.selected = [...state.selected, id];
    chOf(id);
    if (state.playing) startVoice(id);
    else play();
  }
  persist();
}

/** Tells the desktop shell whether sound is playing (close → tray vs. quit). */
function reportPlaying() {
  if (!isTauri()) return;
  void import('@tauri-apps/api/core')
    .then(({ invoke }) => invoke('set_sound_playing', { playing: state.playing }))
    .catch(() => {});
}

export function play() {
  if (state.selected.length === 0) return;
  state.playing = true;
  reportPlaying();
  engine.restoreMaster();
  for (const id of state.selected) startVoice(id);
}

export function pause() {
  state.playing = false;
  reportPlaying();
  engine.stopAll(1.2);
}

export function togglePlay() {
  if (state.playing) pause();
  else play();
}

export function clearMix() {
  pause();
  state.selected = [];
  state.solo = null;
  state.sceneId = null;
  persist();
}

export function setVolume(id: string, v: number) {
  chOf(id).volume = v;
  if (chOf(id).muted && v > 0) chOf(id).muted = false;
  engine.setVolume(id, effective(id));
  persist();
}

export function setPan(id: string, p: number) {
  chOf(id).pan = p;
  engine.setPan(id, p);
  persist();
}

export function toggleMute(id: string) {
  chOf(id).muted = !chOf(id).muted;
  engine.setVolume(id, effective(id));
  persist();
}

export function toggleSolo(id: string) {
  state.solo = state.solo === id ? null : id;
  syncVolumes();
}

export function setMaster(v: number) {
  state.master = v;
  if (v > 0) state.masterMuted = false;
  engine.setMaster(state.masterMuted ? 0 : v);
  persist();
}

export function toggleMasterMute() {
  state.masterMuted = !state.masterMuted;
  engine.setMaster(state.masterMuted ? 0 : state.master);
}

function applyMix(mix: { id: string; volume: number; pan?: number }[]) {
  const ids = mix.map((m) => m.id).filter((id) => SOUND_BY_ID.has(id) || state.custom.some((c) => c.id === id));
  for (const id of state.selected) if (!ids.includes(id)) engine.stop(id);
  for (const m of mix) {
    if (!ids.includes(m.id)) continue;
    state.ch[m.id] = { volume: m.volume, pan: m.pan ?? 0, muted: false };
  }
  state.selected = ids;
  state.solo = null;
  state.playing = true;
  reportPlaying();
  engine.restoreMaster();
  for (const id of ids) startVoice(id);
  persist();
}

/** Plays an Immersive Mode mix (marks it as the current "scene"). */
export function applyModeMix(modeId: string, mix: { id: string; volume: number }[]) {
  applyMix(mix);
  state.sceneId = 'mode:' + modeId;
  persist();
}

export function applyScene(scene: Scene) {
  if (state.sceneId === scene.id && state.playing) {
    pause();
    return;
  }
  applyMix(scene.mix);
  state.sceneId = scene.id;
  persist();
}

export function saveUserMix(name: string) {
  const mix = state.selected.map((id) => ({ id, volume: chOf(id).volume, pan: chOf(id).pan }));
  if (!mix.length) return;
  state.userMixes = [...state.userMixes, { id: `um-${Date.now()}`, name: name.trim() || 'Mix', mix }];
  persist();
}

export function applyUserMix(m: UserMix) {
  applyMix(m.mix);
  state.sceneId = m.id;
  persist();
}

export function deleteUserMix(id: string) {
  state.userMixes = state.userMixes.filter((m) => m.id !== id);
  if (state.sceneId === id) state.sceneId = null;
  persist();
}

// ---------------------------------------------------------------------------
// Sleep timer
// ---------------------------------------------------------------------------

/**
 * What happens when the sleep timer runs out (after the 30 s fade):
 * - pause:    pause the sounds (default)
 * - quit:     close CACalm
 * - sleep:    put the computer to sleep (after a 60 s cancellable countdown)
 * - shutdown: shut the computer down (after a 60 s cancellable countdown).
 *             Never remembered — it resets to "pause" after use or app restart.
 */
export type SleepAction = 'pause' | 'quit' | 'sleep' | 'shutdown';
const POWER_COUNTDOWN_SECS = 60;
const ACTION_KEY = 'cacalm_sleep_action';

export function setSleepAction(action: SleepAction) {
  state.sleepAction = action;
  try {
    // Shutdown is deliberately not persisted.
    localStorage.setItem(ACTION_KEY, action === 'shutdown' ? 'pause' : action);
  } catch {
    /* ignore */
  }
}

export function setSleepTimer(minutes: number, action?: SleepAction) {
  if (action) setSleepAction(action);
  state.sleepMinutes = minutes;
  state.sleepFading = false;
  engine.restoreMaster();
  state.now = Date.now();
  state.sleepEndsAt = minutes > 0 ? Date.now() + minutes * 60_000 : null;
  if (minutes > 0 && !state.playing) play();
}

export function sleepRemaining(): number {
  if (!state.sleepEndsAt) return 0;
  return Math.max(0, Math.ceil((state.sleepEndsAt - state.now) / 1000));
}

let countdownTimer: ReturnType<typeof setInterval> | null = null;

export function cancelPowerAction() {
  if (countdownTimer) clearInterval(countdownTimer);
  countdownTimer = null;
  state.powerCountdown = null;
  if (state.sleepAction === 'shutdown') setSleepAction('pause');
}

async function runPowerAction(action: 'sleep' | 'shutdown') {
  if (!isTauri()) return;
  try {
    const { invoke } = await import('@tauri-apps/api/core');
    await invoke('system_power', { action });
  } catch (e) {
    console.warn('power action failed', e);
  }
}

function startPowerCountdown(action: 'sleep' | 'shutdown') {
  state.powerCountdown = POWER_COUNTDOWN_SECS;
  state.powerCountdownAction = action;
  // The countdown must be visible to be cancellable, even if CACalm is in the tray.
  if (isTauri()) {
    void import('@tauri-apps/api/core').then(({ invoke }) => invoke('window_attention')).catch(() => {});
  }
  countdownTimer = setInterval(() => {
    if (state.powerCountdown === null) return;
    state.powerCountdown -= 1;
    if (state.powerCountdown <= 0) {
      if (countdownTimer) clearInterval(countdownTimer);
      countdownTimer = null;
      state.powerCountdown = null;
      if (action === 'shutdown') setSleepAction('pause');
      void runPowerAction(action);
    }
  }, 1000);
}

async function runEndAction() {
  const action = state.sleepAction;
  if (action === 'quit') {
    if (isTauri()) {
      const { invoke } = await import('@tauri-apps/api/core');
      await invoke('app_quit').catch(() => {});
    }
  } else if (action === 'sleep' || action === 'shutdown') {
    startPowerCountdown(action);
  }
}

function sleepTick() {
  // Only touch reactive state while a sleep timer runs, so an idle CACalm
  // does not re-render the player bar every second.
  if (!state.sleepEndsAt) return;
  state.now = Date.now();
  const left = (state.sleepEndsAt - state.now) / 1000;
  if (!state.sleepFading && left <= SLEEP_FADE_SECS) {
    state.sleepFading = true;
    void engine.fadeOutMaster(Math.max(1, left)).then(() => {
      if (!state.sleepFading) return; // cancelled meanwhile
      pause();
      state.sleepFading = false;
      state.sleepEndsAt = null;
      state.sleepMinutes = 0;
      setTimeout(() => engine.restoreMaster(), 1500);
      void runEndAction();
    });
  }
}

// ---------------------------------------------------------------------------
// Custom (imported) sounds
// ---------------------------------------------------------------------------

interface SoundEntry {
  id: string;
  name: string;
  category: string;
  file_name: string;
}

export async function refreshCustomSounds() {
  if (!isTauri()) return;
  try {
    const { invoke } = await import('@tauri-apps/api/core');
    const list = await invoke<SoundEntry[]>('list_sounds');
    state.custom = list
      .filter((s) => s.category === 'imported')
      .map((s) => {
        const id = `custom:${s.id}`;
        engine.registerCustom(id, () => invoke<ArrayBuffer>('read_sound_file', { id: s.id }));
        return { id, name: s.name };
      });
    const known = new Set(state.custom.map((c) => c.id));
    const stale = state.selected.filter((id) => id.startsWith('custom:') && !known.has(id));
    if (stale.length) {
      for (const id of stale) engine.stop(id);
      state.selected = state.selected.filter((id) => !stale.includes(id));
      persist();
    }
  } catch (e) {
    console.warn('Failed to load custom sounds', e);
  }
}

// ---------------------------------------------------------------------------
// Init (called once from the root layout)
// ---------------------------------------------------------------------------

let initialised = false;

export function initMixer() {
  if (initialised || typeof window === 'undefined') return;
  initialised = true;
  restore();
  try {
    const a = localStorage.getItem(ACTION_KEY);
    if (a === 'quit' || a === 'sleep') state.sleepAction = a;
  } catch {
    /* ignore */
  }
  engine.setLoadingListener((id, loading) => {
    state.loading[id] = loading;
  });
  void refreshCustomSounds();
  setInterval(sleepTick, 1000);
  reportPlaying();
  if (isTauri()) {
    void import('@tauri-apps/api/event').then(({ listen }) => listen('tray-toggle-sound', () => togglePlay()));
  }
}

export function nameOfCustom(id: string): string {
  return state.custom.find((c) => c.id === id)?.name ?? 'Custom';
}
