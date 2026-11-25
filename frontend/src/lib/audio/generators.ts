// Procedural soundscape generators. Each generator renders a seamless stereo
// loop (see dsp.ts for the looping strategy). Pure functions — safe to run in
// a Web Worker.

import {
  Biquad,
  Brown,
  Mod,
  Pink,
  Rng,
  TAU,
  addEvent,
  env,
  eventTimes,
  filterCircular,
  finalize,
  hashSeed,
  loop2,
  panGains,
  reverb,
  whiteArr,
} from './dsp';

type Out = [Float32Array, Float32Array];

interface GenDef {
  /** Loop length in seconds (integer for tonal sounds so they loop exactly). */
  seconds: number;
  /** Loudness target (RMS) after normalisation. */
  rms: number;
  render: (n: number, sr: number, rng: Rng) => Out;
}

const stereo = (n: number): Out => [new Float32Array(n), new Float32Array(n)];

function mixInto(dst: Out, src: Out, gain: number) {
  const [a, b] = dst;
  const [c, d] = src;
  for (let i = 0; i < a.length; i++) {
    a[i] += c[i] * gain;
    b[i] += d[i] * gain;
  }
}

type Color = 'white' | 'pink' | 'brown';

function colorFn(kind: Color): (w: number) => number {
  if (kind === 'pink') {
    const p = new Pink();
    return (w) => p.next(w);
  }
  if (kind === 'brown') {
    const b = new Brown();
    return (w) => b.next(w);
  }
  return (w) => w * 0.35;
}

/** Filtered stereo noise bed. `width` 0 = mono, 1 = fully decorrelated. */
function noiseBed(
  n: number,
  sr: number,
  rng: Rng,
  kind: Color,
  filters: (sr: number) => Biquad[],
  width = 0.85,
  amp?: (k: number) => number
): Out {
  const wl = whiteArr(n, rng);
  const wr = whiteArr(n, rng);
  const out = stereo(n);
  const cl = colorFn(kind);
  const cr = colorFn(kind);
  const fl = filters(sr);
  const fr = filters(sr);
  loop2(n, (k, w) => {
    let l = cl(wl[k]);
    let r = cr(wr[k] * width + wl[k] * (1 - width));
    for (const f of fl) l = f.process(l);
    for (const f of fr) r = f.process(r);
    if (w) {
      const g = amp ? amp(k) : 1;
      out[0][k] = l * g;
      out[1][k] = r * g;
    }
  });
  return out;
}

/** Decaying sinusoid via a rotating phasor (cheap, used for bells & tones). */
function addPartial(
  o: Out,
  start: number,
  sr: number,
  f: number,
  amp: number,
  decaySec: number,
  pan: number,
  attackSec = 0.002,
  maxSec = decaySec * 6
) {
  const n = o[0].length;
  // May exceed n: the tail wraps around so long decays loop seamlessly.
  const len = Math.floor(maxSec * sr);
  const w = (TAU * f) / sr;
  const r = Math.exp(-1 / (decaySec * sr));
  const cw = Math.cos(w) * r;
  const sw = Math.sin(w) * r;
  const aq = Math.exp(-1 / Math.max(1, attackSec * sr));
  const [gl, gr] = panGains(pan);
  let re = amp;
  let im = 0;
  let att = 1;
  let k = ((Math.floor(start) % n) + n) % n;
  for (let j = 0; j < len; j++) {
    const v = im * (1 - att);
    o[0][k] += v * gl;
    o[1][k] += v * gr;
    const nr = re * cw - im * sw;
    im = re * sw + im * cw;
    re = nr;
    att *= aq;
    if (++k === n) k = 0;
  }
}

// ---------------------------------------------------------------------------
// Layers
// ---------------------------------------------------------------------------

interface RainOpts {
  bedHp: number;
  bedLp: number;
  bedGain: number;
  dropRate: number;
  dropFreq: [number, number];
  dropDecayMs: [number, number];
  dropGain: number;
  bigRate: number;
  bigFreq: [number, number];
  bigDecayMs: number;
  lp: number;
}

function rainLayer(n: number, sr: number, rng: Rng, o: RainOpts): Out {
  const intensity = new Mod(n, rng, 1, 4);
  const out = noiseBed(
    n,
    sr,
    rng,
    'pink',
    (s) => [new Biquad(s).hp(o.bedHp), new Biquad(s).lp(o.bedLp)],
    0.8,
    (k) => o.bedGain * (0.85 + 0.15 * intensity.unit(k))
  );
  const drop = (t: number, fr: [number, number], dec: [number, number], gain: number) => {
    const f = Math.exp(rng.range(Math.log(fr[0]), Math.log(fr[1])));
    const d = (rng.range(dec[0], dec[1]) / 1000) * sr;
    const len = Math.floor(d * 6);
    const a = (0.05 + 0.95 * rng.next() ** 3) * gain;
    const w = (TAU * f) / sr;
    addEvent(out[0], out[1], t, len, rng.bi() * 0.9, (j) => {
      return Math.sin(w * j * (1 - (0.12 * j) / len)) * env(j, 0.0004 * sr, d) * a;
    });
  };
  for (const t of eventTimes(n, sr, o.dropRate, rng)) drop(t, o.dropFreq, o.dropDecayMs, o.dropGain);
  for (const t of eventTimes(n, sr, o.bigRate, rng))
    drop(t, o.bigFreq, [o.bigDecayMs * 0.7, o.bigDecayMs * 1.3], o.dropGain * 1.6);
  filterCircular(out[0], out[1], () => [new Biquad(sr).lp(o.lp), new Biquad(sr).hp(40)]);
  return out;
}

interface WindOpts {
  center: [number, number];
  q: number;
  gustMin: number;
  whistle: number;
  rumble: number;
  hiss: number;
}

function windLayer(n: number, sr: number, rng: Rng, o: WindOpts): Out {
  const wl = whiteArr(n, rng);
  const wr = whiteArr(n, rng);
  const cL = new Mod(n, rng, 1, 6);
  const cR = new Mod(n, rng, 1, 6);
  const gust = new Mod(n, rng, 2, 9);
  const bL = new Biquad(sr);
  const bR = new Biquad(sr);
  const whL = new Biquad(sr);
  const whR = new Biquad(sr);
  const ruL = new Biquad(sr).lp(110);
  const ruR = new Biquad(sr).lp(110);
  const hiL = new Biquad(sr).hp(3500);
  const hiR = new Biquad(sr).hp(3500);
  const pL = new Pink();
  const pR = new Pink();
  const brL = new Brown();
  const brR = new Brown();
  const [c0, c1] = o.center;
  const out = stereo(n);
  loop2(n, (k, w) => {
    if (k % 32 === 0) {
      const fl = c0 + (c1 - c0) * cL.unit(k);
      const fr = c0 + (c1 - c0) * cR.unit(k);
      bL.bp(fl, o.q);
      bR.bp(fr, o.q);
      whL.bp(fl * 2.2, 9);
      whR.bp(fr * 2.1, 9);
    }
    const g = o.gustMin + (1 - o.gustMin) * gust.unit(k) ** 1.5;
    const wh = Math.max(0, g - 0.65) * o.whistle;
    const l =
      bL.process(wl[k]) * g +
      whL.process(wl[k]) * wh +
      ruL.process(brL.next(wl[k])) * o.rumble +
      hiL.process(pL.next(wr[k])) * o.hiss * g;
    const r =
      bR.process(wr[k]) * g +
      whR.process(wr[k]) * wh +
      ruR.process(brR.next(wr[k])) * o.rumble +
      hiR.process(pR.next(wl[k])) * o.hiss * g;
    if (w) {
      out[0][k] = l;
      out[1][k] = r;
    }
  });
  return out;
}

type Species = 'warble' | 'whistle' | 'trill' | 'coo';

function birdsLayer(n: number, sr: number, rng: Rng, phraseRate: number, species: Species[], count: number): Out {
  const out = stereo(n);
  const birds = Array.from({ length: count }, () => ({
    pan: rng.bi() * 0.85,
    gain: rng.range(0.25, 1),
    sp: rng.pick(species),
    s: rng.range(0.85, 1.18),
  }));
  interface Note {
    at: number;
    len: number;
    f0: number;
    f1: number;
    amp: number;
    vib?: number;
  }
  for (const t of eventTimes(n, sr, phraseRate, rng)) {
    const b = rng.pick(birds);
    const notes: Note[] = [];
    let at = 0;
    if (b.sp === 'warble') {
      const cnt = rng.int(4, 9);
      for (let i = 0; i < cnt; i++) {
        const len = rng.range(0.05, 0.12);
        const f0 = rng.range(2200, 4200) * b.s;
        notes.push({ at, len, f0, f1: f0 * rng.range(0.7, 1.4), amp: rng.range(0.5, 1) });
        at += len + rng.range(0.025, 0.07);
      }
    } else if (b.sp === 'whistle') {
      notes.push({ at: 0, len: rng.range(0.28, 0.38), f0: 3300 * b.s, f1: 3150 * b.s, amp: 1, vib: 14 });
      notes.push({ at: 0.45, len: rng.range(0.25, 0.35), f0: 2700 * b.s, f1: 2600 * b.s, amp: 0.85, vib: 14 });
    } else if (b.sp === 'trill') {
      const cnt = rng.int(10, 20);
      for (let i = 0; i < cnt; i++) {
        const f0 = (4600 - (700 * i) / cnt) * b.s;
        notes.push({ at: i * 0.045, len: 0.026, f0, f1: f0 * 0.9, amp: 0.8 });
      }
    } else {
      // coo — soft low dove-like call
      const f = rng.range(480, 640) * b.s;
      notes.push({ at: 0, len: 0.32, f0: f * 0.95, f1: f, amp: 0.8 });
      notes.push({ at: 0.45, len: 0.5, f0: f * 1.05, f1: f * 0.92, amp: 1 });
      notes.push({ at: 1.05, len: 0.35, f0: f, f1: f * 0.9, amp: 0.7 });
    }
    for (const nt of notes) {
      const len = Math.floor(nt.len * sr);
      let ph = 0;
      const a = nt.amp * b.gain * (b.sp === 'coo' ? 0.7 : 0.35);
      addEvent(out[0], out[1], t + nt.at * sr, len, b.pan, (j) => {
        const x = j / len;
        const sweep = 0.5 - 0.5 * Math.cos(Math.PI * x);
        const vib = nt.vib ? 40 * Math.sin((TAU * nt.vib * j) / sr) : 0;
        ph += (TAU * (nt.f0 + (nt.f1 - nt.f0) * sweep + vib)) / sr;
        const e = Math.sin(Math.PI * x) ** 2;
        return (Math.sin(ph) + 0.12 * Math.sin(2 * ph)) * e * a;
      });
    }
  }
  return out;
}

function cricketsLayer(n: number, sr: number, rng: Rng, count: number): Out {
  const out = stereo(n);
  for (let c = 0; c < count; c++) {
    const f = rng.range(4200, 5200);
    const period = rng.range(0.45, 0.9) * sr;
    const pulses = rng.int(3, 4);
    const pl = rng.range(0.012, 0.018) * sr;
    const pan = rng.bi() * 0.9;
    const gain = rng.range(0.25, 1);
    const w = (TAU * f) / sr;
    let t = rng.next() * period;
    while (t < n) {
      if (rng.next() < 0.92) {
        for (let p = 0; p < pulses; p++) {
          addEvent(out[0], out[1], t + p * pl * 2, Math.floor(pl), pan, (j) => {
            return Math.sin(w * j) * Math.sin((Math.PI * j) / pl) ** 2 * gain * 0.25;
          });
        }
      }
      t += period * (1 + 0.03 * rng.bi());
    }
  }
  return out;
}

// ---------------------------------------------------------------------------
// Generators
// ---------------------------------------------------------------------------

function noiseColor(kind: Color, filters: (sr: number) => Biquad[], rms: number): GenDef {
  return {
    seconds: 12,
    rms,
    render: (n, sr, rng) => noiseBed(n, sr, rng, kind, filters, 1),
  };
}

function binaural(carrier: number, beat: number): GenDef {
  return {
    seconds: 10,
    rms: 0.07,
    render: (n, sr) => {
      const o = stereo(n);
      for (let k = 0; k < n; k++) {
        o[0][k] = Math.sin((TAU * carrier * k) / sr);
        o[1][k] = Math.sin((TAU * (carrier + beat) * k) / sr);
      }
      return o;
    },
  };
}

function breathing(phases: [number, number, number, number]): GenDef {
  const D = phases[0] + phases[1] + phases[2] + phases[3];
  return {
    seconds: D,
    rms: 0.09,
    render: (n, sr, rng) => {
      const [inh, h1, exh] = phases;
      const level = (t: number) => {
        if (t < inh) return 0.2 + 0.8 * (0.5 - 0.5 * Math.cos((Math.PI * t) / inh));
        if (t < inh + h1) return 1;
        if (t < inh + h1 + exh) return 0.2 + 0.8 * (0.5 + 0.5 * Math.cos((Math.PI * (t - inh - h1)) / exh));
        return 0.2;
      };
      const breath = (t: number) => {
        if (t < inh) return Math.sin((Math.PI * t) / inh) * 0.9;
        if (t >= inh + h1 && t < inh + h1 + exh) return Math.sin((Math.PI * (t - inh - h1)) / exh) * 0.7;
        return 0;
      };
      const det = 1 / D;
      const chord: [number, number][] = [
        [110, 0.5],
        [165, 0.3],
        [220, 0.35],
        [330, 0.15],
        [440, 0.08],
      ];
      const padL = new Float32Array(n);
      const padR = new Float32Array(n);
      for (let k = 0; k < n; k++) {
        let l = 0;
        let r = 0;
        for (const [f, a] of chord) {
          l += a * Math.sin((TAU * (f + det) * k) / sr);
          r += a * Math.sin((TAU * (f - det) * k) / sr);
        }
        padL[k] = l;
        padR[k] = r;
      }
      const wl = whiteArr(n, rng);
      const out = stereo(n);
      const lpL = new Biquad(sr);
      const lpR = new Biquad(sr);
      const pk = new Pink();
      const bp = new Biquad(sr).bp(900, 0.7);
      loop2(n, (k, w) => {
        const t = k / sr;
        const lv = level(t);
        if (k % 32 === 0) {
          const fc = 350 + (2200 * (lv - 0.2)) / 0.8;
          lpL.lp(fc);
          lpR.lp(fc);
        }
        const br = bp.process(pk.next(wl[k])) * breath(t) * 1.4;
        const l = lpL.process(padL[k]) * lv + br;
        const r = lpR.process(padR[k]) * lv + br;
        if (w) {
          out[0][k] = l;
          out[1][k] = r;
        }
      });
      // Gentle cue bells at every phase change.
      const cues: [number, number][] = [
        [0, 660],
        [inh, 880],
        [inh + h1, 550],
      ];
      if (phases[3] > 0) cues.push([inh + h1 + exh, 440]);
      for (const [at, f] of cues) {
        addPartial(out, at * sr, sr, f, 0.12, 1.1, 0, 0.004);
        addPartial(out, at * sr, sr, f * 2.76, 0.03, 0.4, 0, 0.004);
      }
      return out;
    },
  };
}

export const GENERATORS: Record<string, GenDef> = {
  // ------------------------------ Nature ------------------------------
  'ch-rain': {
    seconds: 24,
    rms: 0.12,
    render: (n, sr, rng) =>
      rainLayer(n, sr, rng, {
        bedHp: 400,
        bedLp: 7000,
        bedGain: 1,
        dropRate: 150,
        dropFreq: [1800, 5200],
        dropDecayMs: [3, 7],
        dropGain: 0.18,
        bigRate: 4,
        bigFreq: [700, 1500],
        bigDecayMs: 18,
        lp: 9000,
      }),
  },
  'n-storm': {
    seconds: 40,
    rms: 0.13,
    render: (n, sr, rng) => {
      const o = rainLayer(n, sr, rng, {
        bedHp: 250,
        bedLp: 6000,
        bedGain: 1.3,
        dropRate: 220,
        dropFreq: [1500, 4500],
        dropDecayMs: [3, 8],
        dropGain: 0.18,
        bigRate: 8,
        bigFreq: [500, 1200],
        bigDecayMs: 22,
        lp: 7000,
      });
      const strikes = 2;
      for (let s = 0; s < strikes; s++) {
        const start = ((s + rng.range(0.1, 0.8)) / strikes) * n;
        const len = Math.floor(rng.range(7, 10) * sr);
        const lp = new Biquad(sr).lp(rng.range(140, 240));
        const br = new Brown();
        const subs = [0, rng.range(0.6, 1.6), rng.range(2, 3.4)].map((x) => x * sr);
        const decs = subs.map(() => rng.range(1.2, 2.6) * sr);
        const amps = [1, rng.range(0.5, 0.9), rng.range(0.3, 0.6)];
        const amp = rng.range(2.2, 3);
        addEvent(o[0], o[1], start, len, rng.bi() * 0.5, (j) => {
          let e = 0;
          for (let i = 0; i < 3; i++) {
            const d = j - subs[i];
            if (d > 0) e += amps[i] * (1 - Math.exp(-d / (0.25 * sr))) * Math.exp(-d / decs[i]);
          }
          return lp.process(br.next(rng.bi())) * e * amp;
        });
      }
      return o;
    },
  },
  'ch-waves': {
    seconds: 36,
    rms: 0.12,
    render: (n, sr, rng) => {
      const intervals: number[] = [];
      let sum = 0;
      while (sum < n) {
        const v = rng.range(7.5, 11.5) * sr;
        intervals.push(v);
        sum += v;
      }
      const scale = n / sum;
      const envA = new Float32Array(n);
      let t = rng.next() * sr;
      for (const iv of intervals) {
        const rise = rng.range(2.2, 3.5) * sr;
        const dec = rng.range(2.8, 4.5) * sr;
        const amp = rng.range(0.6, 1);
        const len = Math.floor(rise + dec * 5);
        for (let j = 0; j < len; j++) {
          const x = j < rise ? 0.5 - 0.5 * Math.cos((Math.PI * j) / rise) : Math.exp(-(j - rise) / dec);
          envA[Math.floor(t + j) % n] += amp * x;
        }
        t += iv * scale;
      }
      const shift = Math.floor(0.35 * sr);
      const wl = whiteArr(n, rng);
      const wr = whiteArr(n, rng);
      const pL = new Pink();
      const pR = new Pink();
      const bL = new Brown();
      const bR = new Brown();
      const lpL = new Biquad(sr);
      const lpR = new Biquad(sr);
      const ruL = new Biquad(sr).lp(180);
      const ruR = new Biquad(sr).lp(180);
      const foL = new Biquad(sr).hp(2500);
      const foR = new Biquad(sr).hp(2500);
      const out = stereo(n);
      loop2(n, (k, w) => {
        const eL = Math.min(1.2, envA[k]);
        const eR = Math.min(1.2, envA[(k - shift + n) % n]);
        if (k % 32 === 0) {
          lpL.lp(250 + 3200 * eL ** 1.7, 0.6);
          lpR.lp(250 + 3200 * eR ** 1.7, 0.6);
        }
        const pl = pL.next(wl[k]);
        const pr = pR.next(wr[k]);
        const l =
          lpL.process(pl) * (0.08 + eL) +
          ruL.process(bL.next(wl[k])) * (0.35 + 0.4 * eL) * 0.6 +
          foL.process(pl) * Math.max(0, eL - 0.5) * 0.3;
        const r =
          lpR.process(pr) * (0.08 + eR) +
          ruR.process(bR.next(wr[k])) * (0.35 + 0.4 * eR) * 0.6 +
          foR.process(pr) * Math.max(0, eR - 0.5) * 0.3;
        if (w) {
          out[0][k] = l;
          out[1][k] = r;
        }
      });
      return out;
    },
  },
  'ch-stream': {
    seconds: 24,
    rms: 0.11,
    render: (n, sr, rng) => {
      const D = n / sr;
      const m1 = new Mod(n, rng, D * 3, D * 8, 6);
      const m2 = new Mod(n, rng, D * 3, D * 8, 6);
      const o = noiseBed(n, sr, rng, 'white', (s) => [new Biquad(s).bp(900, 0.6)], 0.9, (k) => 0.4 + 0.6 * m1.unit(k));
      const hi = noiseBed(n, sr, rng, 'white', (s) => [new Biquad(s).bp(2600, 0.9)], 0.9, (k) => 0.3 + 0.7 * m2.unit(k));
      mixInto(o, hi, 0.6);
      for (const t of eventTimes(n, sr, 35, rng)) {
        const f0 = Math.exp(rng.range(Math.log(350), Math.log(1800)));
        const d = rng.range(0.004, 0.015) * sr;
        const len = Math.floor(d * 5);
        const a = (0.05 + 0.4 * rng.next() ** 2) * 0.5;
        let ph = 0;
        addEvent(o[0], o[1], t, len, rng.bi() * 0.8, (j) => {
          ph += (TAU * f0 * (1 + (1.8 * j) / len)) / sr;
          return Math.sin(ph) * env(j, 0.0005 * sr, d) * a;
        });
      }
      return o;
    },
  },
  'ch-wind': {
    seconds: 30,
    rms: 0.11,
    render: (n, sr, rng) =>
      windLayer(n, sr, rng, { center: [280, 850], q: 1.6, gustMin: 0.3, whistle: 0.35, rumble: 0.35, hiss: 0.05 }),
  },
  'ch-birds': {
    seconds: 40,
    rms: 0.09,
    render: (n, sr, rng) => {
      const breeze = new Mod(n, rng, 2, 8);
      const o = noiseBed(n, sr, rng, 'pink', (s) => [new Biquad(s).bp(3000, 0.5)], 0.9, (k) => 0.12 * (0.4 + 0.6 * breeze.unit(k)));
      mixInto(o, noiseBed(n, sr, rng, 'brown', (s) => [new Biquad(s).lp(200)], 0.8), 0.08);
      mixInto(o, birdsLayer(n, sr, rng, 0.45, ['warble', 'whistle', 'trill', 'warble'], 6), 1);
      reverb(o[0], o[1], sr, { wet: 0.18, size: 0.7, damp: 0.5 });
      return o;
    },
  },
  'n-crickets': {
    seconds: 24,
    rms: 0.08,
    render: (n, sr, rng) => {
      const o = noiseBed(n, sr, rng, 'brown', (s) => [new Biquad(s).lp(280)], 0.8);
      for (let i = 0; i < n; i++) {
        o[0][i] *= 0.15;
        o[1][i] *= 0.15;
      }
      mixInto(o, cricketsLayer(n, sr, rng, 5), 1);
      reverb(o[0], o[1], sr, { wet: 0.12, size: 0.6 });
      return o;
    },
  },
  'n-frogs': {
    seconds: 30,
    rms: 0.09,
    render: (n, sr, rng) => {
      const o = noiseBed(n, sr, rng, 'brown', (s) => [new Biquad(s).lp(250)], 0.8);
      for (let i = 0; i < n; i++) {
        o[0][i] *= 0.12;
        o[1][i] *= 0.12;
      }
      mixInto(o, cricketsLayer(n, sr, rng, 3), 0.35);
      for (let fr = 0; fr < 3; fr++) {
        const f0 = rng.range(120, 260);
        const pr = rng.range(25, 40);
        const pan = rng.bi() * 0.8;
        const gain = rng.range(0.5, 1);
        const F = rng.range(550, 900);
        let t = rng.next() * 3 * sr;
        while (t < n) {
          const croaks = rng.int(1, 3);
          for (let c = 0; c < croaks; c++) {
            const len = Math.floor(rng.range(0.15, 0.35) * sr);
            addEvent(o[0], o[1], t + c * 0.38 * sr, len, pan, (j) => {
              const tt = j / sr;
              let s = 0;
              for (let h = 1; h <= 8; h++) {
                const fh = h * f0;
                s += Math.exp(-(((fh - F) / 400) ** 2)) * Math.sin(TAU * fh * tt);
              }
              const am = Math.max(0, Math.sin(Math.PI * pr * tt)) ** 4;
              return s * am * Math.sin((Math.PI * j) / len) * gain * 0.25;
            });
          }
          t += rng.range(2, 6) * sr;
        }
      }
      reverb(o[0], o[1], sr, { wet: 0.15, size: 0.6 });
      return o;
    },
  },
  'n-jungle': {
    seconds: 40,
    rms: 0.1,
    render: (n, sr, rng) => {
      const o = noiseBed(n, sr, rng, 'pink', (s) => [new Biquad(s).lp(900)], 0.8);
      for (let i = 0; i < n; i++) {
        o[0][i] *= 0.3;
        o[1][i] *= 0.3;
      }
      const D = n / sr;
      for (let c = 0; c < 3; c++) {
        const f = rng.range(4500, 7500);
        const fam = Math.round(rng.range(40, 120) * D) / D;
        const swell = new Mod(n, rng, 1, 4);
        const pan = rng.bi() * 0.8;
        const ins = noiseBed(n, sr, rng, 'white', (s) => [new Biquad(s).bp(f, 6)], 0.2, (k) => {
          const am = (0.5 + 0.5 * Math.sin((TAU * fam * k) / sr)) ** 2;
          return am * swell.unit(k) ** 2;
        });
        const [gl, gr] = panGains(pan);
        for (let i = 0; i < n; i++) {
          o[0][i] += ins[0][i] * gl * 0.5;
          o[1][i] += ins[1][i] * gr * 0.5;
        }
      }
      mixInto(o, birdsLayer(n, sr, rng, 0.7, ['whistle', 'trill', 'coo', 'warble'], 7), 1);
      reverb(o[0], o[1], sr, { wet: 0.2, size: 0.75 });
      return o;
    },
  },
  'n-cave': {
    seconds: 30,
    rms: 0.08,
    render: (n, sr, rng) => {
      const o = noiseBed(n, sr, rng, 'brown', (s) => [new Biquad(s).lp(150)], 0.8);
      for (let i = 0; i < n; i++) {
        o[0][i] *= 0.1;
        o[1][i] *= 0.1;
      }
      for (const t of eventTimes(n, sr, 0.7, rng)) {
        const f0 = rng.range(600, 1500);
        const d = rng.range(0.018, 0.03) * sr;
        const a = rng.range(0.4, 1);
        let ph = 0;
        addEvent(o[0], o[1], t, Math.floor(d * 6), rng.bi() * 0.8, (j) => {
          ph += (TAU * f0 * (1 + 0.7 * Math.min(1, j / (0.025 * sr)))) / sr;
          return Math.sin(ph) * env(j, 0.0005 * sr, d) * a;
        });
      }
      reverb(o[0], o[1], sr, { wet: 0.6, size: 0.95, damp: 0.2 });
      return o;
    },
  },
  'n-snow': {
    seconds: 30,
    rms: 0.09,
    render: (n, sr, rng) =>
      windLayer(n, sr, rng, { center: [200, 500], q: 0.8, gustMin: 0.5, whistle: 0.12, rumble: 0.25, hiss: 0.12 }),
  },

  // ------------------------------ Sleep ------------------------------
  'ch-noise': noiseColor('brown', (s) => [new Biquad(s).hp(25)], 0.14),
  's-pink': noiseColor('pink', (s) => [new Biquad(s).hp(30)], 0.11),
  's-white': noiseColor('white', (s) => [new Biquad(s).lp(9000), new Biquad(s).hp(40)], 0.08),
  's-green': noiseColor('pink', (s) => [new Biquad(s).hp(150), new Biquad(s).lp(1200)], 0.12),
  's-fan': {
    seconds: 16,
    rms: 0.12,
    render: (n, sr, rng) => {
      const o = noiseBed(n, sr, rng, 'brown', (s) => [new Biquad(s).lp(700)], 0.6);
      mixInto(o, noiseBed(n, sr, rng, 'pink', (s) => [new Biquad(s).lp(2200)], 0.6), 0.35);
      for (let k = 0; k < n; k++) {
        const am = 1 + 0.05 * Math.sin((TAU * 18 * k) / sr);
        const hum = 0.006 * Math.sin((TAU * 120 * k) / sr) + 0.003 * Math.sin((TAU * 240 * k) / sr);
        o[0][k] = o[0][k] * am + hum;
        o[1][k] = o[1][k] * am + hum;
      }
      return o;
    },
  },
  's-airplane': {
    seconds: 20,
    rms: 0.13,
    render: (n, sr, rng) => {
      const m = new Mod(n, rng, 1, 3);
      const o = noiseBed(n, sr, rng, 'brown', (s) => [new Biquad(s).lp(300)], 0.5, (k) => 0.95 + 0.05 * m.unit(k));
      mixInto(o, noiseBed(n, sr, rng, 'pink', (s) => [new Biquad(s).lp(1200)], 0.5), 0.25);
      mixInto(o, noiseBed(n, sr, rng, 'pink', (s) => [new Biquad(s).bp(450, 2)], 0.5), 0.1);
      return o;
    },
  },
  's-train': {
    seconds: 24,
    rms: 0.12,
    render: (n, sr, rng) => {
      const sway = new Mod(n, rng, 2, 6);
      const o = noiseBed(n, sr, rng, 'brown', (s) => [new Biquad(s).lp(180)], 0.6, (k) => 0.85 + 0.15 * sway.unit(k));
      mixInto(o, noiseBed(n, sr, rng, 'pink', (s) => [new Biquad(s).bp(500, 0.6)], 0.6), 0.4);
      const cl = stereo(n);
      const period = 1.2 * sr;
      const count = Math.round(n / period);
      for (let i = 0; i < count; i++) {
        for (const [off, amp] of [
          [0, 0.9],
          [0.16, 0.7],
        ] as const) {
          const a = amp * rng.range(0.85, 1);
          const t0 = i * (n / count) + off * sr;
          addEvent(cl[0], cl[1], t0, Math.floor(0.012 * sr), 0, (j) => rng.bi() * Math.exp(-j / (0.0015 * sr)) * a);
          addPartial(o, t0, sr, 85, a * 0.35, 0.035, 0, 0.002);
        }
      }
      filterCircular(cl[0], cl[1], () => [new Biquad(sr).hp(400), new Biquad(sr).lp(3500)]);
      mixInto(o, cl, 0.5);
      return o;
    },
  },
  's-heartbeat': {
    seconds: 20,
    rms: 0.11,
    render: (n, sr, rng) => {
      const o = noiseBed(n, sr, rng, 'brown', (s) => [new Biquad(s).lp(220)], 0.6);
      for (let i = 0; i < n; i++) {
        o[0][i] *= 0.25;
        o[1][i] *= 0.25;
      }
      const beats = Math.round(n / sr);
      for (let b = 0; b < beats; b++) {
        const t0 = b * sr;
        for (const [off, fa, fb, dec, amp] of [
          [0, 75, 45, 0.06, 1],
          [0.33, 95, 60, 0.045, 0.65],
        ] as const) {
          let ph = 0;
          const len = Math.floor(dec * 6 * sr);
          addEvent(o[0], o[1], t0 + off * sr, len, 0, (j) => {
            const f = fb + (fa - fb) * Math.exp(-j / (0.03 * sr));
            ph += (TAU * f) / sr;
            return Math.sin(ph) * env(j, 0.003 * sr, dec * sr) * amp;
          });
        }
      }
      filterCircular(o[0], o[1], () => [new Biquad(sr).lp(320)]);
      return o;
    },
  },
  's-roof': {
    seconds: 24,
    rms: 0.12,
    render: (n, sr, rng) => {
      const o = rainLayer(n, sr, rng, {
        bedHp: 120,
        bedLp: 2200,
        bedGain: 1,
        dropRate: 70,
        dropFreq: [300, 900],
        dropDecayMs: [12, 30],
        dropGain: 0.22,
        bigRate: 4,
        bigFreq: [150, 300],
        bigDecayMs: 40,
        lp: 3500,
      });
      mixInto(o, noiseBed(n, sr, rng, 'brown', (s) => [new Biquad(s).lp(200)], 0.6), 0.15);
      return o;
    },
  },
  's-delta': binaural(100, 2),

  // ------------------------------ Focus ------------------------------
  'ch-cafe': {
    seconds: 40,
    rms: 0.11,
    render: (n, sr, rng) => {
      const VOWELS: [number, number][] = [
        [700, 1200],
        [500, 1900],
        [320, 2300],
        [500, 900],
        [360, 800],
      ];
      const o = stereo(n);
      for (let v = 0; v < 7; v++) {
        const f0 = rng.range(95, 230);
        const pan = rng.range(-0.8, 0.8);
        const gain = rng.range(0.4, 1);
        let t = rng.next() * 2 * sr;
        while (t < n) {
          const boutEnd = t + rng.range(1.5, 4) * sr;
          while (t < boutEnd) {
            const len = Math.floor(rng.range(0.12, 0.25) * sr);
            const [F1, F2] = rng.pick(VOWELS);
            const b1 = new Biquad(sr).bp(F1 * rng.range(0.9, 1.1), F1 / 90);
            const b2 = new Biquad(sr).bp(F2 * rng.range(0.9, 1.1), F2 / 120);
            const pitch = f0 * rng.range(0.92, 1.1);
            let ph = 0;
            addEvent(o[0], o[1], t, len, pan, (j) => {
              ph += pitch / sr;
              const saw = 2 * (ph % 1) - 1;
              const e = Math.sin((Math.PI * j) / len) ** 1.5;
              return (b1.process(saw) + 0.6 * b2.process(saw)) * e * gain;
            });
            t += sr / rng.range(3.5, 6);
          }
          t += rng.range(0.5, 3) * sr;
        }
      }
      filterCircular(o[0], o[1], () => [new Biquad(sr).lp(2200), new Biquad(sr).hp(90)]);
      for (const t of eventTimes(n, sr, 0.35, rng)) {
        const f0 = rng.range(2200, 3400);
        const a = rng.range(0.15, 0.35);
        const pan = rng.bi() * 0.7;
        const s = rng.range(0.6, 1.2);
        addPartial(o, t, sr, f0, a, 0.25 * s, pan);
        addPartial(o, t, sr, f0 * 2.32, a * 0.5, 0.12 * s, pan);
        addPartial(o, t, sr, f0 * 4.1, a * 0.25, 0.06 * s, pan);
      }
      for (const t of eventTimes(n, sr, 0.15, rng)) addPartial(o, t, sr, 140, 0.3, 0.025, rng.bi() * 0.6);
      reverb(o[0], o[1], sr, { wet: 0.45, size: 0.85, damp: 0.5 });
      mixInto(o, noiseBed(n, sr, rng, 'pink', (s) => [new Biquad(s).lp(500)], 0.7), 0.15);
      return o;
    },
  },
  'f-keyboard': {
    seconds: 30,
    rms: 0.08,
    render: (n, sr, rng) => {
      const o = stereo(n);
      let t = rng.next() * sr;
      while (t < n) {
        const words = rng.int(3, 9);
        for (let w = 0; w < words && t < n; w++) {
          const keys = rng.int(2, 8);
          for (let k = 0; k < keys; k++) {
            const fk = rng.range(1800, 3200);
            const a = rng.range(0.4, 1);
            const pan = (k % 2 ? 0.2 : -0.2) + rng.bi() * 0.05;
            addEvent(o[0], o[1], t, Math.floor(0.012 * sr), pan, (j) => {
              return (
                rng.bi() * Math.exp(-j / (0.0012 * sr)) * 0.6 * a +
                Math.sin((TAU * fk * j) / sr) * Math.exp(-j / (0.004 * sr)) * 0.4 * a
              );
            });
            t += rng.range(0.09, 0.16) * sr;
          }
          addEvent(o[0], o[1], t, Math.floor(0.02 * sr), 0, (j) => {
            return Math.sin((TAU * 900 * j) / sr) * Math.exp(-j / (0.006 * sr)) * 0.5;
          });
          t += rng.range(0.1, 0.25) * sr;
        }
        t += rng.range(1, 4) * sr;
      }
      filterCircular(o[0], o[1], () => [new Biquad(sr).lp(5500), new Biquad(sr).hp(150)]);
      reverb(o[0], o[1], sr, { wet: 0.12, size: 0.4 });
      mixInto(o, noiseBed(n, sr, rng, 'pink', (s) => [new Biquad(s).lp(400)], 0.7), 0.05);
      return o;
    },
  },
  'f-library': {
    seconds: 40,
    rms: 0.07,
    render: (n, sr, rng) => {
      const o = noiseBed(n, sr, rng, 'pink', (s) => [new Biquad(s).lp(300)], 0.7);
      for (let i = 0; i < n; i++) {
        o[0][i] *= 0.5;
        o[1][i] *= 0.5;
      }
      mixInto(o, noiseBed(n, sr, rng, 'brown', (s) => [new Biquad(s).lp(100)], 0.6), 0.4);
      const ev = stereo(n);
      for (const t of eventTimes(n, sr, 0.18, rng)) {
        const len = Math.floor(rng.range(0.35, 0.6) * sr);
        const bp = new Biquad(sr);
        const a = rng.range(0.25, 0.5);
        addEvent(ev[0], ev[1], t, len, rng.bi() * 0.7, (j) => {
          if (j % 32 === 0) bp.bp(700 + (2500 * j) / len, 1.2);
          const e = Math.sin((Math.PI * j) / len) ** 2;
          return bp.process(rng.bi() * (0.6 + 0.4 * Math.abs(rng.bi()))) * e * a;
        });
      }
      const steps = rng.int(6, 10);
      const s0 = rng.next() * n;
      for (let s = 0; s < steps; s++) {
        const a = Math.sin((Math.PI * (s + 0.5)) / steps) * 0.5;
        addPartial(ev, s0 + s * 0.55 * sr, sr, 90, a, 0.03, -0.6 + (1.2 * s) / steps);
      }
      mixInto(o, ev, 1);
      reverb(o[0], o[1], sr, { wet: 0.35, size: 0.9, damp: 0.6 });
      return o;
    },
  },
  'f-clock': {
    seconds: 20,
    rms: 0.06,
    render: (n, sr) => {
      const o = stereo(n);
      const ticks = Math.round(n / sr);
      for (let s = 0; s < ticks; s++) {
        const tick = s % 2 === 0;
        const parts = tick ? [2650, 4180, 6100] : [2050, 3300, 5200];
        const t0 = s * sr;
        parts.forEach((f, i) => addPartial(o, t0, sr, f, 0.5 / (i + 1), 0.006, 0, 0.0003));
        addPartial(o, t0, sr, 950, 0.4, 0.018, 0, 0.0005);
      }
      reverb(o[0], o[1], sr, { wet: 0.15, size: 0.3 });
      return o;
    },
  },
  'f-beta': binaural(200, 14),
  'f-gamma': binaural(200, 40),
  'f-iso': {
    seconds: 10,
    rms: 0.07,
    render: (n, sr) => {
      const o = stereo(n);
      for (let k = 0; k < n; k++) {
        const x = ((k * 10) / sr) % 1;
        const e = x < 0.5 ? Math.sin(Math.PI * (x / 0.5)) ** 2 : 0;
        const v = Math.sin((TAU * 220 * k) / sr) * e;
        o[0][k] = v;
        o[1][k] = v;
      }
      return o;
    },
  },

  // ------------------------------ Relax ------------------------------
  'ch-fire': {
    seconds: 30,
    rms: 0.11,
    render: (n, sr, rng) => {
      const D = n / sr;
      const flick = new Mod(n, rng, D * 0.5, D * 2, 5);
      const cluster = new Mod(n, rng, D * 0.1, D * 0.5, 4);
      const o = noiseBed(n, sr, rng, 'brown', (s) => [new Biquad(s).lp(300)], 0.7, (k) => 0.65 + 0.35 * flick.unit(k));
      mixInto(o, noiseBed(n, sr, rng, 'pink', (s) => [new Biquad(s).bp(1800, 0.5)], 0.8, (k) => 0.7 + 0.3 * flick.unit(k)), 0.12);
      const cr = stereo(n);
      for (const t of eventTimes(n, sr, 22, rng)) {
        if (rng.next() > 0.3 + 0.7 * cluster.unit(Math.floor(t))) continue;
        const len = Math.floor(rng.range(0.001, 0.005) * sr);
        const a = 0.1 + 0.9 * rng.next() ** 3;
        addEvent(cr[0], cr[1], t, len * 3, rng.bi() * 0.5, (j) => rng.bi() * Math.exp(-j / (len / 4)) * a);
      }
      filterCircular(cr[0], cr[1], () => [new Biquad(sr).hp(1200), new Biquad(sr).lp(7000)]);
      mixInto(o, cr, 1.6);
      for (const t of eventTimes(n, sr, 0.4, rng)) {
        const a = rng.range(0.6, 1) * 0.5;
        const pan = rng.bi() * 0.4;
        addPartial(o, t, sr, rng.range(180, 320), a, 0.02, pan);
        addEvent(o[0], o[1], t, Math.floor(0.008 * sr), pan, (j) => rng.bi() * Math.exp(-j / (0.0015 * sr)) * a);
      }
      return o;
    },
  },
  'ch-binaural': binaural(200, 10),
  'r-breath478': breathing([4, 7, 8, 0]),
  'r-breathbox': breathing([4, 4, 4, 4]),
  'ch-spatial': {
    seconds: 32,
    rms: 0.09,
    render: (n, sr, rng) => {
      const det = 1 / 32;
      const chord: [number, number][] = [
        [110, 0.5],
        [165, 0.3],
        [220, 0.35],
        [275, 0.15],
        [330, 0.12],
      ];
      const pad = new Float32Array(n);
      for (let k = 0; k < n; k++) {
        let v = 0;
        for (const [f, a] of chord) v += a * (Math.sin((TAU * (f + det) * k) / sr) + Math.sin((TAU * (f - det) * k) / sr)) * 0.5;
        pad[k] = v;
      }
      const wl = whiteArr(n, rng);
      const pk = new Pink();
      const tex = new Biquad(sr).bp(700, 0.5);
      const lp = new Biquad(sr);
      const out = stereo(n);
      loop2(n, (k, w) => {
        const th = (TAU * 2 * k) / n;
        if (k % 32 === 0) lp.lp(900 + 2500 * (0.5 + 0.5 * Math.cos(th)));
        const v = lp.process(pad[k] + tex.process(pk.next(wl[k])) * 0.4);
        if (w) {
          const [gl, gr] = panGains(Math.sin(th));
          out[0][k] = v * gl;
          out[1][k] = v * gr;
        }
      });
      return out;
    },
  },

  // ------------------------------ Meditation ------------------------------
  'm-bowl': {
    seconds: 36,
    rms: 0.09,
    render: (n, sr, rng) => {
      const o = stereo(n);
      const strikes = 3;
      const modes: [number, number, number][] = [
        [1, 1, 9],
        [2.76, 0.5, 5],
        [5.04, 0.25, 2.5],
        [8.2, 0.12, 1.2],
      ];
      const f0 = 196;
      for (let s = 0; s < strikes; s++) {
        const t0 = (s * n) / strikes;
        const a = [1, 0.8, 0.9][s];
        for (const [ratio, amp, dec] of modes) {
          const f = f0 * ratio;
          addPartial(o, t0, sr, f * 1.0025, a * amp * 0.5, dec, -0.5, 0.01);
          addPartial(o, t0, sr, f * 0.9975, a * amp * 0.5, dec, 0.5, 0.01);
        }
        addEvent(o[0], o[1], t0, Math.floor(0.02 * sr), 0, (j) => rng.bi() * Math.exp(-j / (0.003 * sr)) * 0.15 * a);
      }
      reverb(o[0], o[1], sr, { wet: 0.25, size: 0.85 });
      return o;
    },
  },
  'm-chimes': {
    seconds: 40,
    rms: 0.09,
    render: (n, sr, rng) => {
      const o = windLayer(n, sr, rng, { center: [300, 700], q: 1.2, gustMin: 0.3, whistle: 0, rumble: 0.2, hiss: 0.03 });
      for (let i = 0; i < n; i++) {
        o[0][i] *= 0.35;
        o[1][i] *= 0.35;
      }
      const gust = new Mod(n, rng, 2, 8);
      const tubes = [1047, 1175, 1319, 1568, 1760].map((f, i) => ({ f, pan: -0.7 + i * 0.35 }));
      for (const t of eventTimes(n, sr, 1.2, rng)) {
        if (rng.next() > gust.unit(Math.floor(t)) ** 2) continue;
        const tb = rng.pick(tubes);
        const a = rng.range(0.2, 0.7);
        addPartial(o, t, sr, tb.f, a, 3.5, tb.pan, 0.001);
        addPartial(o, t, sr, tb.f * 2.76, a * 0.35, 1.6, tb.pan, 0.001);
        addPartial(o, t, sr, tb.f * 5.4, a * 0.15, 0.7, tb.pan, 0.001);
      }
      reverb(o[0], o[1], sr, { wet: 0.18, size: 0.7 });
      return o;
    },
  },
  'm-drone': {
    seconds: 8,
    rms: 0.08,
    render: (n, sr) => {
      const o = stereo(n);
      const parts: [number, number][] = [
        [108, 0.55],
        [216, 0.4],
        [324, 0.18],
        [432, 0.3],
        [648, 0.1],
        [864, 0.06],
      ];
      for (let k = 0; k < n; k++) {
        const br = 0.8 + 0.2 * Math.sin((TAU * k) / n);
        let l = 0;
        let r = 0;
        for (const [f, a] of parts) {
          l += a * Math.sin((TAU * (f + 0.125) * k) / sr);
          r += a * Math.sin((TAU * (f - 0.125) * k) / sr);
        }
        o[0][k] = l * br;
        o[1][k] = r * br;
      }
      return o;
    },
  },
  'm-theta': binaural(150, 6),
};

export const GENERATOR_IDS = Object.keys(GENERATORS);

/** Renders a sound at the given sample rate. Returns normalised stereo data. */
export function renderSound(id: string, sr: number): Out {
  const def = GENERATORS[id];
  if (!def) throw new Error(`unknown sound: ${id}`);
  const n = Math.round(def.seconds * sr);
  const rng = new Rng(hashSeed(id));
  const out = def.render(n, sr, rng);
  finalize(out[0], out[1], def.rms);
  return out;
}
