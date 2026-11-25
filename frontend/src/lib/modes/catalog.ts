// Immersive Modes are pure data: a visual renderer + a sound mix + behaviour.
// New modes can be added here without new code (only new renderers need code).
// Format is the base for future user-made / imported modes (see notes docs/10).

import type { RendererId } from '$lib/visuals/renderers';
import type { L10n } from '$lib/audio/catalog';
import type { SleepAction } from '$lib/stores/mixer.svelte';

export interface ModeDef {
  id: string;
  icon: string;
  renderer: RendererId;
  name: L10n;
  desc: L10n;
  /** Card accent (CSS gradient) on the Sounds page. */
  accent: string;
  mix: { id: string; volume: number }[];
  /** Start a sleep timer on entry. */
  sleep?: { minutes: number; action: SleepAction };
  /** Dim the screen gradually after N minutes, reaching full dim after `overMin`. */
  dim?: { afterMin: number; overMin: number };
  /** Power nap with a gentle wake-up (choices in minutes). */
  nap?: { choices: number[]; wakeMix: { id: string; volume: number }[] };
}

export const MODES: ModeDef[] = [
  {
    id: 'romantic',
    icon: 'heart',
    renderer: 'romantic',
    name: { en: 'Romantic', id: 'Romantis' },
    desc: { en: 'Candlelight, soft rain and a music box', id: 'Cahaya lilin, hujan lembut & music box' },
    accent: 'linear-gradient(135deg,#3b0f22,#8a2c45 55%,#d97745)',
    mix: [
      { id: 'ch-fire', volume: 0.4 },
      { id: 'ch-rain', volume: 0.22 },
      { id: 'r-musicbox', volume: 0.38 },
      { id: 'r-warmpad', volume: 0.22 },
    ],
  },
  {
    id: 'night',
    icon: 'moon',
    renderer: 'night',
    name: { en: 'Night Sleep', id: 'Tidur Malam' },
    desc: { en: 'Starry sky that slowly fades to dark', id: 'Langit berbintang yang meredup perlahan' },
    accent: 'linear-gradient(135deg,#030616,#13204d 55%,#3b4a8a)',
    mix: [
      { id: 'ch-noise', volume: 0.3 },
      { id: 'n-crickets', volume: 0.14 },
      { id: 's-delta', volume: 0.1 },
    ],
    sleep: { minutes: 45, action: 'pause' },
    dim: { afterMin: 3, overMin: 10 },
  },
  {
    id: 'nap',
    icon: 'sparkles',
    renderer: 'nap',
    name: { en: 'Power Nap', id: 'Tidur Siang' },
    desc: { en: '20–30 min rest, then a gentle wake-up', id: 'Istirahat 20–30 mnt, lalu bangun lembut' },
    accent: 'linear-gradient(135deg,#9fb3e6,#e7b9d3 55%,#f8cfae)',
    mix: [
      { id: 's-pink', volume: 0.22 },
      { id: 'ch-waves', volume: 0.3 },
      { id: 'r-warmpad', volume: 0.15 },
    ],
    nap: { choices: [20, 30], wakeMix: [{ id: 'ch-birds', volume: 0.45 }, { id: 'ch-stream', volume: 0.25 }] },
  },
  {
    id: 'forest',
    icon: 'trees',
    renderer: 'forest',
    name: { en: 'Dark Forest', id: 'Hutan Gelap' },
    desc: { en: 'Fireflies, mist and night creatures', id: 'Kunang-kunang, kabut & suara malam' },
    accent: 'linear-gradient(135deg,#010806,#0b2a20 55%,#3f6b2a)',
    mix: [
      { id: 'n-crickets', volume: 0.42 },
      { id: 'n-frogs', volume: 0.28 },
      { id: 'ch-wind', volume: 0.22 },
    ],
  },
  {
    id: 'hearth',
    icon: 'flame',
    renderer: 'hearth',
    name: { en: 'Hearth Fire', id: 'Api Tungku' },
    desc: { en: 'Crackling fire while snow blows outside', id: 'Api berderak, salju bertiup di luar' },
    accent: 'linear-gradient(135deg,#0c0503,#5a1d0a 55%,#e0782a)',
    mix: [
      { id: 'ch-fire', volume: 0.6 },
      { id: 'n-snow', volume: 0.28 },
    ],
  },
];

export const MODE_BY_ID = new Map(MODES.map((m) => [m.id, m]));
