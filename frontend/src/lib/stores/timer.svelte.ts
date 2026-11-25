// Global Pomodoro timer state. The authoritative timer lives in Rust; the UI
// advances it with wall-clock deltas so throttled background timers can't
// make it drift.

import { isTauri } from '$lib/env';
import { engine } from '$lib/audio/engine';

export interface TimerSnapshot {
  phase: string; // FOCUS | SHORT BREAK | LONG BREAK
  remaining: number;
  paused: boolean;
  sessions_today: number;
  focus_seconds_today: number;
}

export const PHASE_SECONDS: Record<string, number> = {
  FOCUS: 25 * 60,
  'SHORT BREAK': 5 * 60,
  'LONG BREAK': 15 * 60,
};

const state = $state<TimerSnapshot>({
  phase: 'FOCUS',
  remaining: 25 * 60,
  paused: true,
  sessions_today: 0,
  focus_seconds_today: 0,
});

const CHIME_KEY = 'cacalm_chime';
const prefs = $state({ chime: true });

export function getTimer() {
  return state;
}

export function getTimerPrefs() {
  return prefs;
}

export function setChime(on: boolean) {
  prefs.chime = on;
  try {
    localStorage.setItem(CHIME_KEY, on ? '1' : '0');
  } catch {
    /* ignore */
  }
}

async function call(cmd: string, args?: Record<string, unknown>): Promise<TimerSnapshot | null> {
  if (!isTauri()) return null;
  try {
    const { invoke } = await import('@tauri-apps/api/core');
    return await invoke<TimerSnapshot>(cmd, args);
  } catch {
    return null;
  }
}

function apply(s: TimerSnapshot) {
  const phaseChanged = s.phase !== state.phase;
  Object.assign(state, s);
  if (phaseChanged && prefs.chime) engine.chime();
}

function localAdvance() {
  const order = state.phase === 'FOCUS' ? 'SHORT BREAK' : 'FOCUS';
  if (state.phase === 'FOCUS') state.sessions_today += 1;
  apply({ ...state, phase: order, remaining: PHASE_SECONDS[order] });
}

export async function toggleTimer() {
  const s = await call(state.paused ? 'timer_resume' : 'timer_pause');
  if (s) apply(s);
  else state.paused = !state.paused;
  lastTs = Date.now();
}

export async function resetTimer() {
  const s = await call('timer_reset');
  if (s) apply(s);
  else Object.assign(state, { remaining: PHASE_SECONDS[state.phase], paused: true });
}

export async function skipPhase() {
  const s = await call('timer_skip');
  if (s) apply(s);
  else localAdvance();
}

let lastTs = Date.now();
let carry = 0;
let started = false;

export function initTimer() {
  if (started || typeof window === 'undefined') return;
  started = true;
  try {
    prefs.chime = localStorage.getItem(CHIME_KEY) !== '0';
  } catch {
    /* ignore */
  }
  void call('get_timer_snapshot').then((s) => s && Object.assign(state, s));
  if (isTauri()) {
    void import('@tauri-apps/api/event').then(({ listen }) =>
      listen<TimerSnapshot>('timer-updated', (e) => {
        apply(e.payload);
        lastTs = Date.now();
      })
    );
  }
  setInterval(async () => {
    const now = Date.now();
    const delta = (now - lastTs) / 1000 + carry;
    lastTs = now;
    if (state.paused) {
      carry = 0;
      return;
    }
    const whole = Math.floor(delta);
    carry = delta - whole;
    if (whole <= 0) return;
    const s = await call('timer_tick', { elapsedSecs: whole });
    if (s) apply(s);
    else if (state.remaining > whole) state.remaining -= whole;
    else localAdvance();
  }, 1000);
}

export function phaseProgress(): number {
  const total = PHASE_SECONDS[state.phase] ?? 25 * 60;
  return Math.max(0, Math.min(1, (total - state.remaining) / total));
}

export function formatClock(secs: number): string {
  const m = Math.floor(secs / 60);
  const s = Math.floor(secs % 60);
  return `${m}:${s < 10 ? '0' : ''}${s}`;
}
