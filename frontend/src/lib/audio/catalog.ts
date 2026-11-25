// Built-in sound catalogue, grouped by purpose. Every entry maps to a
// procedural generator in generators.ts (no audio files are shipped).

export type CategoryId = 'nature' | 'sleep' | 'focus' | 'relax' | 'meditate' | 'mine';

export interface L10n {
  en: string;
  id: string;
}

export interface SoundMeta {
  id: string;
  category: CategoryId;
  icon: string;
  name: L10n;
  desc: L10n;
  /** Suggested starting volume. */
  volume: number;
  /** Needs headphones to work (binaural beats). */
  headphones?: boolean;
  /** Breathing guide: phase lengths in seconds [inhale, hold, exhale, hold]. */
  breath?: [number, number, number, number];
}

export const CATEGORIES: { id: CategoryId; icon: string; name: L10n; desc: L10n }[] = [
  {
    id: 'nature',
    icon: 'leaf',
    name: { en: 'Nature', id: 'Alam' },
    desc: { en: 'Rain, ocean, forest and night sounds', id: 'Hujan, laut, hutan & suasana malam' },
  },
  {
    id: 'sleep',
    icon: 'moon',
    name: { en: 'Sleep', id: 'Tidur' },
    desc: { en: 'Steady noise and soft rhythms to drift off', id: 'Noise stabil & ritme lembut pengantar tidur' },
  },
  {
    id: 'focus',
    icon: 'target',
    name: { en: 'Focus', id: 'Fokus' },
    desc: { en: 'Workspace ambience and focus frequencies', id: 'Suasana kerja & frekuensi konsentrasi' },
  },
  {
    id: 'relax',
    icon: 'sparkles',
    name: { en: 'Relax', id: 'Relaks' },
    desc: { en: 'Breathing guides, warmth and alpha waves', id: 'Pemandu napas, kehangatan & gelombang alfa' },
  },
  {
    id: 'meditate',
    icon: 'lotus',
    name: { en: 'Meditate', id: 'Meditasi' },
    desc: { en: 'Bowls, chimes and deep drones', id: 'Mangkuk, lonceng & dengung dalam' },
  },
];

export const SOUNDS: SoundMeta[] = [
  // Nature
  { id: 'ch-rain', category: 'nature', icon: 'rain', volume: 0.6, name: { en: 'Rain', id: 'Hujan' }, desc: { en: 'Steady gentle rainfall', id: 'Hujan lembut yang stabil' } },
  { id: 'n-storm', category: 'nature', icon: 'storm', volume: 0.55, name: { en: 'Thunderstorm', id: 'Badai Petir' }, desc: { en: 'Heavy rain with distant thunder', id: 'Hujan deras & guntur di kejauhan' } },
  { id: 'ch-waves', category: 'nature', icon: 'waves', volume: 0.55, name: { en: 'Ocean Waves', id: 'Ombak Laut' }, desc: { en: 'Slow waves rolling onto the shore', id: 'Ombak pelan menyapu pantai' } },
  { id: 'ch-stream', category: 'nature', icon: 'droplets', volume: 0.5, name: { en: 'Stream', id: 'Sungai' }, desc: { en: 'Babbling brook over stones', id: 'Gemericik air sungai di bebatuan' } },
  { id: 'ch-wind', category: 'nature', icon: 'wind', volume: 0.45, name: { en: 'Wind', id: 'Angin' }, desc: { en: 'Gusts across open hills', id: 'Hembusan angin di perbukitan' } },
  { id: 'ch-birds', category: 'nature', icon: 'bird', volume: 0.5, name: { en: 'Forest Birds', id: 'Burung Hutan' }, desc: { en: 'Morning songbirds in the trees', id: 'Kicau burung pagi di pepohonan' } },
  { id: 'n-crickets', category: 'nature', icon: 'cricket', volume: 0.45, name: { en: 'Crickets', id: 'Jangkrik' }, desc: { en: 'Warm summer night', id: 'Malam musim panas yang hangat' } },
  { id: 'n-frogs', category: 'nature', icon: 'frog', volume: 0.45, name: { en: 'Frogs', id: 'Katak' }, desc: { en: 'Pond chorus after rain', id: 'Paduan suara katak selepas hujan' } },
  { id: 'n-jungle', category: 'nature', icon: 'trees', volume: 0.45, name: { en: 'Tropical Forest', id: 'Hutan Tropis' }, desc: { en: 'Insects and exotic birds', id: 'Serangga & burung eksotis' } },
  { id: 'n-cave', category: 'nature', icon: 'mountain', volume: 0.5, name: { en: 'Cave Drips', id: 'Tetesan Gua' }, desc: { en: 'Echoing water drops', id: 'Tetes air bergema di gua' } },
  { id: 'n-snow', category: 'nature', icon: 'snow', volume: 0.45, name: { en: 'Snowy Wind', id: 'Angin Salju' }, desc: { en: 'Soft wind over fresh snow', id: 'Angin lembut di atas salju' } },

  // Sleep
  { id: 'ch-noise', category: 'sleep', icon: 'noise', volume: 0.45, name: { en: 'Brown Noise', id: 'Brown Noise' }, desc: { en: 'Deep, warm rumble', id: 'Gemuruh dalam & hangat' } },
  { id: 's-pink', category: 'sleep', icon: 'noise', volume: 0.4, name: { en: 'Pink Noise', id: 'Pink Noise' }, desc: { en: 'Balanced, like steady rain', id: 'Seimbang, seperti hujan stabil' } },
  { id: 's-white', category: 'sleep', icon: 'noise', volume: 0.3, name: { en: 'White Noise', id: 'White Noise' }, desc: { en: 'Soft hiss that masks noise', id: 'Desis halus penutup kebisingan' } },
  { id: 's-green', category: 'sleep', icon: 'noise', volume: 0.4, name: { en: 'Green Noise', id: 'Green Noise' }, desc: { en: 'Mid-range, nature-like calm', id: 'Frekuensi tengah, tenang alami' } },
  { id: 's-fan', category: 'sleep', icon: 'fan', volume: 0.45, name: { en: 'Fan', id: 'Kipas Angin' }, desc: { en: 'Familiar bedroom fan hum', id: 'Dengung kipas kamar yang akrab' } },
  { id: 's-airplane', category: 'sleep', icon: 'plane', volume: 0.45, name: { en: 'Airplane Cabin', id: 'Kabin Pesawat' }, desc: { en: 'Cruising altitude drone', id: 'Dengung pesawat saat terbang' } },
  { id: 's-train', category: 'sleep', icon: 'train', volume: 0.45, name: { en: 'Night Train', id: 'Kereta Malam' }, desc: { en: 'Rhythmic rails and rumble', id: 'Ritme rel & gemuruh kereta' } },
  { id: 's-heartbeat', category: 'sleep', icon: 'heart', volume: 0.45, name: { en: 'Heartbeat', id: 'Detak Jantung' }, desc: { en: 'Slow 60 bpm comfort', id: 'Detak pelan 60 bpm menenangkan' } },
  { id: 's-roof', category: 'sleep', icon: 'home', volume: 0.55, name: { en: 'Rain on Roof', id: 'Hujan di Atap' }, desc: { en: 'Muffled rain from indoors', id: 'Hujan teredam dari dalam rumah' } },
  { id: 's-delta', category: 'sleep', icon: 'wave', volume: 0.3, headphones: true, name: { en: 'Delta 2 Hz', id: 'Delta 2 Hz' }, desc: { en: 'Binaural · deep sleep', id: 'Binaural · tidur nyenyak' } },

  // Focus
  { id: 'ch-cafe', category: 'focus', icon: 'coffee', volume: 0.45, name: { en: 'Coffee Shop', id: 'Kedai Kopi' }, desc: { en: 'Soft chatter and cups', id: 'Obrolan samar & denting cangkir' } },
  { id: 'f-keyboard', category: 'focus', icon: 'keyboard', volume: 0.4, name: { en: 'Typing', id: 'Ketikan Keyboard' }, desc: { en: 'Someone working nearby', id: 'Seseorang bekerja di dekatmu' } },
  { id: 'f-library', category: 'focus', icon: 'book', volume: 0.5, name: { en: 'Quiet Library', id: 'Perpustakaan' }, desc: { en: 'Hushed hall, turning pages', id: 'Ruang hening & lembar halaman' } },
  { id: 'f-clock', category: 'focus', icon: 'clock', volume: 0.35, name: { en: 'Ticking Clock', id: 'Detak Jam' }, desc: { en: 'Steady 1 second rhythm', id: 'Ritme stabil tiap detik' } },
  { id: 'f-beta', category: 'focus', icon: 'wave', volume: 0.3, headphones: true, name: { en: 'Beta 14 Hz', id: 'Beta 14 Hz' }, desc: { en: 'Binaural · alert focus', id: 'Binaural · fokus waspada' } },
  { id: 'f-gamma', category: 'focus', icon: 'wave', volume: 0.25, headphones: true, name: { en: 'Gamma 40 Hz', id: 'Gamma 40 Hz' }, desc: { en: 'Binaural · deep concentration', id: 'Binaural · konsentrasi dalam' } },
  { id: 'f-iso', category: 'focus', icon: 'pulse', volume: 0.25, name: { en: 'Isochronic 10 Hz', id: 'Isokronik 10 Hz' }, desc: { en: 'Pulsed tone, no headphones', id: 'Nada berdenyut, tanpa headphone' } },

  // Relax
  { id: 'ch-fire', category: 'relax', icon: 'flame', volume: 0.55, name: { en: 'Fireplace', id: 'Perapian' }, desc: { en: 'Crackling wood fire', id: 'Kayu bakar berderak hangat' } },
  { id: 'ch-binaural', category: 'relax', icon: 'wave', volume: 0.3, headphones: true, name: { en: 'Alpha 10 Hz', id: 'Alfa 10 Hz' }, desc: { en: 'Binaural · calm alertness', id: 'Binaural · tenang namun sadar' } },
  { id: 'r-breath478', category: 'relax', icon: 'breath', volume: 0.5, breath: [4, 7, 8, 0], name: { en: 'Breathe 4-7-8', id: 'Napas 4-7-8' }, desc: { en: 'Guided calming breath', id: 'Panduan napas penenang' } },
  { id: 'r-breathbox', category: 'relax', icon: 'square', volume: 0.5, breath: [4, 4, 4, 4], name: { en: 'Box Breathing', id: 'Napas Kotak' }, desc: { en: '4-4-4-4 steady rhythm', id: 'Ritme stabil 4-4-4-4' } },
  { id: 'r-musicbox', category: 'relax', icon: 'music', volume: 0.4, name: { en: 'Music Box', id: 'Music Box' }, desc: { en: 'Soft arpeggios, slow and warm', id: 'Arpeggio lembut, pelan & hangat' } },
  { id: 'r-warmpad', category: 'relax', icon: 'sparkles', volume: 0.35, name: { en: 'Warm Pad', id: 'Pad Hangat' }, desc: { en: 'Slow major-7 chords', id: 'Akor mayor-7 yang pelan' } },
  { id: 'ch-spatial', category: 'relax', icon: 'headphones', volume: 0.4, name: { en: '8D Ambient', id: 'Ambient 8D' }, desc: { en: 'Warm pad circling around you', id: 'Nada hangat berputar di sekitarmu' } },

  // Meditate
  { id: 'm-bowl', category: 'meditate', icon: 'bowl', volume: 0.5, name: { en: 'Singing Bowl', id: 'Singing Bowl' }, desc: { en: 'Resonant bowl every 12 s', id: 'Mangkuk berdengung tiap 12 detik' } },
  { id: 'm-chimes', category: 'meditate', icon: 'bell', volume: 0.45, name: { en: 'Wind Chimes', id: 'Lonceng Angin' }, desc: { en: 'Pentatonic chimes in a breeze', id: 'Lonceng pentatonik tertiup angin' } },
  { id: 'm-drone', category: 'meditate', icon: 'infinity', volume: 0.4, name: { en: 'Drone 432 Hz', id: 'Dengung 432 Hz' }, desc: { en: 'Slow, grounding harmonic drone', id: 'Dengung harmonik yang membumikan' } },
  { id: 'm-theta', category: 'meditate', icon: 'wave', volume: 0.3, headphones: true, name: { en: 'Theta 6 Hz', id: 'Theta 6 Hz' }, desc: { en: 'Binaural · deep meditation', id: 'Binaural · meditasi dalam' } },
];

export const SOUND_BY_ID = new Map(SOUNDS.map((s) => [s.id, s]));

export interface Scene {
  id: string;
  icon: string;
  name: L10n;
  desc: L10n;
  mix: { id: string; volume: number }[];
}

export const SCENES: Scene[] = [
  {
    id: 'sc-sleep',
    icon: 'moon',
    name: { en: 'Deep Sleep', id: 'Tidur Nyenyak' },
    desc: { en: 'Roof rain, brown noise, delta', id: 'Hujan di atap, brown noise, delta' },
    mix: [
      { id: 's-roof', volume: 0.6 },
      { id: 'ch-noise', volume: 0.3 },
      { id: 's-delta', volume: 0.15 },
    ],
  },
  {
    id: 'sc-focus',
    icon: 'target',
    name: { en: 'Deep Focus', id: 'Fokus Dalam' },
    desc: { en: 'Rain, brown noise, beta', id: 'Hujan, brown noise, beta' },
    mix: [
      { id: 'ch-rain', volume: 0.5 },
      { id: 'ch-noise', volume: 0.3 },
      { id: 'f-beta', volume: 0.15 },
    ],
  },
  {
    id: 'sc-cafe',
    icon: 'coffee',
    name: { en: 'Rainy Café', id: 'Kafe Hujan' },
    desc: { en: 'Coffee shop on a rainy day', id: 'Kedai kopi saat hujan' },
    mix: [
      { id: 'ch-cafe', volume: 0.55 },
      { id: 'ch-rain', volume: 0.3 },
    ],
  },
  {
    id: 'sc-calm',
    icon: 'waves',
    name: { en: 'Calm Shore', id: 'Pantai Tenang' },
    desc: { en: 'Waves, chimes, alpha', id: 'Ombak, lonceng, alfa' },
    mix: [
      { id: 'ch-waves', volume: 0.6 },
      { id: 'm-chimes', volume: 0.25 },
      { id: 'ch-binaural', volume: 0.15 },
    ],
  },
  {
    id: 'sc-breathe',
    icon: 'breath',
    name: { en: 'Breathe', id: 'Bernapas' },
    desc: { en: '4-7-8 breathing by the sea', id: 'Napas 4-7-8 di tepi laut' },
    mix: [
      { id: 'r-breath478', volume: 0.55 },
      { id: 'ch-waves', volume: 0.3 },
    ],
  },
  {
    id: 'sc-meditate',
    icon: 'lotus',
    name: { en: 'Meditation', id: 'Meditasi' },
    desc: { en: 'Bowl, drone, stream', id: 'Mangkuk, dengung, sungai' },
    mix: [
      { id: 'm-bowl', volume: 0.5 },
      { id: 'm-drone', volume: 0.25 },
      { id: 'ch-stream', volume: 0.3 },
    ],
  },
  {
    id: 'sc-night',
    icon: 'trees',
    name: { en: 'Forest Night', id: 'Malam di Hutan' },
    desc: { en: 'Crickets, frogs, campfire', id: 'Jangkrik, katak, api unggun' },
    mix: [
      { id: 'n-crickets', volume: 0.45 },
      { id: 'n-frogs', volume: 0.3 },
      { id: 'ch-fire', volume: 0.4 },
    ],
  },
  {
    id: 'sc-cabin',
    icon: 'storm',
    name: { en: 'Storm Cabin', id: 'Pondok Badai' },
    desc: { en: 'Thunderstorm by the fire', id: 'Badai petir di dekat perapian' },
    mix: [
      { id: 'n-storm', volume: 0.55 },
      { id: 'ch-fire', volume: 0.45 },
    ],
  },
];

export function loc(l: L10n, locale: string): string {
  return locale === 'id' ? l.id : l.en;
}
