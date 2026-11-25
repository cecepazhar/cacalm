// Small offline DSP toolkit used to render seamless, procedurally generated
// ambient loops. Everything here is pure and framework-free so it can run in
// a Web Worker.
//
// Seamless looping strategy: every sound is rendered as a *circular* signal of
// `n` samples. Noise sources are pre-generated arrays indexed modulo `n`, and
// discrete events (drops, chirps, crackles...) are written with wrap-around.
// Stateful processing (filters, reverb) is run over two periods and only the
// second period is kept, so the filter state at the loop seam is already in
// steady state and the loop has no click, gap or audible crossfade.

export const TAU = Math.PI * 2;

export class Rng {
  private s: number;
  constructor(seed: number) {
    this.s = seed >>> 0 || 0x9e3779b9;
  }
  /** Uniform [0, 1) */
  next(): number {
    let x = this.s;
    x ^= x << 13;
    x ^= x >>> 17;
    x ^= x << 5;
    this.s = x >>> 0;
    return this.s / 4294967296;
  }
  /** Uniform [-1, 1) */
  bi(): number {
    return this.next() * 2 - 1;
  }
  range(a: number, b: number): number {
    return a + (b - a) * this.next();
  }
  int(a: number, b: number): number {
    return Math.floor(this.range(a, b + 1));
  }
  pick<T>(arr: readonly T[]): T {
    return arr[Math.floor(this.next() * arr.length)];
  }
  /** Exponentially distributed interval with the given mean. */
  exp(mean: number): number {
    return -Math.log(1 - this.next()) * mean;
  }
}

export function hashSeed(id: string): number {
  let h = 0x811c9dc5;
  for (let i = 0; i < id.length; i++) {
    h ^= id.charCodeAt(i);
    h = Math.imul(h, 0x01000193);
  }
  return h >>> 0;
}

/** RBJ biquad, transposed direct form II. */
export class Biquad {
  private b0 = 1;
  private b1 = 0;
  private b2 = 0;
  private a1 = 0;
  private a2 = 0;
  private z1 = 0;
  private z2 = 0;
  private sr: number;
  constructor(sr: number) {
    this.sr = sr;
  }

  private prep(f: number, q: number) {
    const fc = Math.min(Math.max(f, 10), this.sr * 0.45);
    const w = (TAU * fc) / this.sr;
    return { c: Math.cos(w), al: Math.sin(w) / (2 * Math.max(q, 0.05)) };
  }
  lp(f: number, q = 0.7071): this {
    const { c, al } = this.prep(f, q);
    const a0 = 1 + al;
    this.b0 = (1 - c) / 2 / a0;
    this.b1 = (1 - c) / a0;
    this.b2 = this.b0;
    this.a1 = (-2 * c) / a0;
    this.a2 = (1 - al) / a0;
    return this;
  }
  hp(f: number, q = 0.7071): this {
    const { c, al } = this.prep(f, q);
    const a0 = 1 + al;
    this.b0 = (1 + c) / 2 / a0;
    this.b1 = -(1 + c) / a0;
    this.b2 = this.b0;
    this.a1 = (-2 * c) / a0;
    this.a2 = (1 - al) / a0;
    return this;
  }
  /** Band-pass with 0 dB peak gain. */
  bp(f: number, q = 1): this {
    const { c, al } = this.prep(f, q);
    const a0 = 1 + al;
    this.b0 = al / a0;
    this.b1 = 0;
    this.b2 = -al / a0;
    this.a1 = (-2 * c) / a0;
    this.a2 = (1 - al) / a0;
    return this;
  }
  process(x: number): number {
    const y = this.b0 * x + this.z1;
    this.z1 = this.b1 * x - this.a1 * y + this.z2;
    this.z2 = this.b2 * x - this.a2 * y;
    return y;
  }
}

/** Paul Kellet's refined pink noise filter. */
export class Pink {
  private b0 = 0;
  private b1 = 0;
  private b2 = 0;
  private b3 = 0;
  private b4 = 0;
  private b5 = 0;
  private b6 = 0;
  next(w: number): number {
    this.b0 = 0.99886 * this.b0 + w * 0.0555179;
    this.b1 = 0.99332 * this.b1 + w * 0.0750759;
    this.b2 = 0.969 * this.b2 + w * 0.153852;
    this.b3 = 0.8665 * this.b3 + w * 0.3104856;
    this.b4 = 0.55 * this.b4 + w * 0.5329522;
    this.b5 = -0.7616 * this.b5 - w * 0.016898;
    const out = this.b0 + this.b1 + this.b2 + this.b3 + this.b4 + this.b5 + this.b6 + w * 0.5362;
    this.b6 = w * 0.115926;
    return out * 0.11;
  }
}

/** Leaky-integrated (red / brown) noise. */
export class Brown {
  private l = 0;
  next(w: number): number {
    this.l = (this.l + 0.02 * w) / 1.02;
    return this.l * 3.5;
  }
}

export function whiteArr(n: number, rng: Rng): Float32Array {
  const a = new Float32Array(n);
  for (let i = 0; i < n; i++) a[i] = rng.bi();
  return a;
}

/** Runs `fn` for two periods; `write` is true only during the second one. */
export function loop2(n: number, fn: (k: number, write: boolean) => void) {
  for (let i = 0; i < n * 2; i++) fn(i % n, i >= n);
}

/**
 * Smooth, loop-periodic random modulation in roughly [-1, 1]. Built from a sum
 * of sinusoids with an integer number of cycles over the loop, evaluated at a
 * control rate and linearly interpolated.
 */
export class Mod {
  private arr: Float32Array;
  private step: number;
  constructor(n: number, rng: Rng, minCycles: number, maxCycles: number, partials = 4) {
    this.step = 64;
    const m = Math.ceil(n / this.step) + 2;
    this.arr = new Float32Array(m);
    const parts: { c: number; p: number; a: number }[] = [];
    let total = 0;
    for (let j = 0; j < partials; j++) {
      const c = Math.max(1, Math.round(rng.range(minCycles, maxCycles)));
      const a = 1 / (1 + j * 0.6);
      parts.push({ c, p: rng.next() * TAU, a });
      total += a;
    }
    for (let i = 0; i < m; i++) {
      const ph = ((i * this.step) % n) / n;
      let v = 0;
      for (const pt of parts) v += pt.a * Math.sin(TAU * pt.c * ph + pt.p);
      this.arr[i] = v / total;
    }
  }
  get(k: number): number {
    const x = k / this.step;
    const i = Math.floor(x);
    const f = x - i;
    return this.arr[i] * (1 - f) + this.arr[i + 1] * f;
  }
  /** Mapped to [0, 1]. */
  unit(k: number): number {
    return Math.min(1, Math.max(0, this.get(k) * 0.5 + 0.5));
  }
}

/** Constant-power pan gains for p in [-1, 1]. */
export function panGains(p: number): [number, number] {
  const th = ((Math.max(-1, Math.min(1, p)) + 1) * Math.PI) / 4;
  return [Math.cos(th), Math.sin(th)];
}

/** Adds a mono event into a stereo circular buffer with constant-power pan. */
export function addEvent(
  L: Float32Array,
  R: Float32Array,
  start: number,
  len: number,
  pan: number,
  gen: (j: number) => number
) {
  const n = L.length;
  const [gl, gr] = panGains(pan);
  let k = ((Math.floor(start) % n) + n) % n;
  for (let j = 0; j < len; j++) {
    const v = gen(j);
    L[k] += v * gl;
    R[k] += v * gr;
    if (++k === n) k = 0;
  }
}

/** Soft attack / exponential decay envelope. */
export function env(j: number, attack: number, decay: number): number {
  return (1 - Math.exp(-j / Math.max(attack, 1e-6))) * Math.exp(-j / decay);
}

/**
 * Poisson-ish event times across a loop of `n` samples.
 * `rate` is events per second.
 */
export function eventTimes(n: number, sr: number, rate: number, rng: Rng): number[] {
  const out: number[] = [];
  const mean = sr / Math.max(rate, 1e-6);
  let t = rng.next() * mean;
  while (t < n) {
    out.push(t);
    t += rng.exp(mean);
  }
  return out;
}

/** Applies a time-invariant filter chain to a circular stereo signal in place. */
export function filterCircular(L: Float32Array, R: Float32Array, make: () => Biquad[]) {
  const n = L.length;
  const fl = make();
  const fr = make();
  const oL = new Float32Array(n);
  const oR = new Float32Array(n);
  loop2(n, (k, w) => {
    let a = L[k];
    let b = R[k];
    for (const f of fl) a = f.process(a);
    for (const f of fr) b = f.process(b);
    if (w) {
      oL[k] = a;
      oR[k] = b;
    }
  });
  L.set(oL);
  R.set(oR);
}

// ---------------------------------------------------------------------------
// Freeverb-style stereo reverb (circular, two-pass)
// ---------------------------------------------------------------------------

class Comb {
  private buf: Float32Array;
  private i = 0;
  private store = 0;
  private fb: number;
  private damp: number;
  constructor(len: number, fb: number, damp: number) {
    this.fb = fb;
    this.damp = damp;
    this.buf = new Float32Array(len);
  }
  process(x: number): number {
    const out = this.buf[this.i];
    this.store = out * (1 - this.damp) + this.store * this.damp;
    this.buf[this.i] = x + this.store * this.fb;
    if (++this.i >= this.buf.length) this.i = 0;
    return out;
  }
}

class Allpass {
  private buf: Float32Array;
  private i = 0;
  constructor(len: number) {
    this.buf = new Float32Array(len);
  }
  process(x: number): number {
    const b = this.buf[this.i];
    this.buf[this.i] = x + b * 0.5;
    if (++this.i >= this.buf.length) this.i = 0;
    return b - x;
  }
}

const COMB_T = [1116, 1188, 1277, 1356, 1422, 1491, 1557, 1617];
const AP_T = [556, 441, 341, 225];

export function reverb(
  L: Float32Array,
  R: Float32Array,
  sr: number,
  opts: { size?: number; damp?: number; wet: number; dry?: number }
) {
  const n = L.length;
  const scale = sr / 44100;
  const fb = 0.7 + 0.28 * (opts.size ?? 0.8);
  const damp = (opts.damp ?? 0.4) * 0.4;
  const mk = (spread: number) => ({
    combs: COMB_T.map((t) => new Comb(Math.round((t + spread) * scale), fb, damp)),
    aps: AP_T.map((t) => new Allpass(Math.round((t + spread) * scale))),
  });
  const l = mk(0);
  const r = mk(23);
  const oL = new Float32Array(n);
  const oR = new Float32Array(n);
  const dry = opts.dry ?? 1;
  loop2(n, (k, w) => {
    const input = (L[k] + R[k]) * 0.015;
    let a = 0;
    let b = 0;
    for (const c of l.combs) a += c.process(input);
    for (const c of r.combs) b += c.process(input);
    for (const ap of l.aps) a = ap.process(a);
    for (const ap of r.aps) b = ap.process(b);
    if (w) {
      oL[k] = L[k] * dry + a * opts.wet;
      oR[k] = R[k] * dry + b * opts.wet;
    }
  });
  L.set(oL);
  R.set(oR);
}

/**
 * Removes DC, scales to a target RMS and tames transient peaks with a soft-knee
 * limiter so sparse sounds (frogs, typing, clock) are not much quieter than
 * dense beds.
 */
export function finalize(L: Float32Array, R: Float32Array, targetRms: number, peakMax = 0.89) {
  const n = L.length;
  let mL = 0;
  let mR = 0;
  for (let i = 0; i < n; i++) {
    mL += L[i];
    mR += R[i];
  }
  mL /= n;
  mR /= n;
  let sum = 0;
  let peak = 1e-9;
  for (let i = 0; i < n; i++) {
    L[i] -= mL;
    R[i] -= mR;
    sum += L[i] * L[i] + R[i] * R[i];
    peak = Math.max(peak, Math.abs(L[i]), Math.abs(R[i]));
  }
  const rms = Math.sqrt(sum / (2 * n)) || 1e-9;
  // Allow up to ~6 dB of peak reduction by the limiter below.
  const g = Math.min(targetRms / rms, (peakMax * 2) / peak);
  const knee = peakMax * 0.65;
  const room = peakMax - knee;
  const lim = (x: number) => {
    const a = Math.abs(x);
    if (a <= knee) return x;
    return Math.sign(x) * (knee + room * Math.tanh((a - knee) / room));
  };
  for (let i = 0; i < n; i++) {
    L[i] = lim(L[i] * g);
    R[i] = lim(R[i] * g);
  }
}
