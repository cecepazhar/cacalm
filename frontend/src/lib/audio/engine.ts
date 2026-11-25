// CACalm playback engine (Web Audio API). Single source of truth for sound —
// the native backend is intentionally not used, so nothing plays twice.
//
// Graph per voice:  BufferSource(loop) → StereoPanner → Gain ─┐
//                                                           master Gain → Compressor → destination
//
// All volume changes are ramped to avoid zipper noise / clicks.

const GEN_RATE = 32000;
// Rendered loops kept in memory (each 2–10 MB). Everything not playing is released
// TRIM_AFTER_MS after playback stops.
const MAX_CACHED = 4;
const TRIM_AFTER_MS = 120_000;
const WORKER_IDLE_MS = 60_000;

type Loader = () => Promise<ArrayBuffer>;

interface Voice {
  src: AudioBufferSourceNode;
  gain: GainNode;
  pan: StereoPannerNode;
  startedAt: number;
  duration: number;
}

class CalmEngine {
  private ctx: AudioContext | null = null;
  private master: GainNode | null = null;
  private voices = new Map<string, Voice>();
  private cache = new Map<string, AudioBuffer>();
  private pending = new Map<string, Promise<AudioBuffer>>();
  private wanted = new Map<string, { volume: number; pan: number }>();
  private customLoaders = new Map<string, Loader>();
  private worker: Worker | null = null;
  private workerFailed = false;
  private reqId = 0;
  private waiters = new Map<number, { resolve: (v: [Float32Array, Float32Array]) => void; reject: (e: Error) => void }>();
  private masterVolume = 0.8;
  private onLoading: ((id: string, loading: boolean) => void) | null = null;
  private suspendTimer: ReturnType<typeof setTimeout> | null = null;
  private trimTimer: ReturnType<typeof setTimeout> | null = null;
  private workerIdleTimer: ReturnType<typeof setTimeout> | null = null;

  setLoadingListener(fn: (id: string, loading: boolean) => void) {
    this.onLoading = fn;
  }

  registerCustom(id: string, loader: Loader) {
    this.customLoaders.set(id, loader);
  }

  private ensure(): AudioContext {
    if (!this.ctx) {
      const Ctx =
        window.AudioContext ||
        (window as unknown as { webkitAudioContext: typeof AudioContext }).webkitAudioContext;
      this.ctx = new Ctx({ latencyHint: 'playback' });
      this.master = this.ctx.createGain();
      this.master.gain.value = this.masterVolume;
      const comp = this.ctx.createDynamicsCompressor();
      comp.threshold.value = -14;
      comp.knee.value = 12;
      comp.ratio.value = 4;
      comp.attack.value = 0.01;
      comp.release.value = 0.3;
      this.master.connect(comp);
      comp.connect(this.ctx.destination);
    }
    if (this.ctx.state === 'suspended') void this.ctx.resume();
    return this.ctx;
  }

  private getWorker(): Worker | null {
    if (this.worker || this.workerFailed) return this.worker;
    try {
      this.worker = new Worker(new URL('./synth.worker.ts', import.meta.url), { type: 'module' });
      this.worker.onmessage = (e: MessageEvent) => {
        const { reqId, L, R, error } = e.data as {
          reqId: number;
          L?: Float32Array;
          R?: Float32Array;
          error?: string;
        };
        const w = this.waiters.get(reqId);
        if (!w) return;
        this.waiters.delete(reqId);
        this.armWorkerIdle();
        if (error || !L || !R) w.reject(new Error(error ?? 'render failed'));
        else w.resolve([L, R]);
      };
      this.worker.onerror = () => {
        this.workerFailed = true;
        this.worker = null;
        for (const w of this.waiters.values()) w.reject(new Error('worker failed'));
        this.waiters.clear();
      };
    } catch {
      this.workerFailed = true;
      this.worker = null;
    }
    return this.worker;
  }

/** Terminates the render worker after a minute without work (it holds ~10–20 MB). */
  private armWorkerIdle() {
    if (this.workerIdleTimer) clearTimeout(this.workerIdleTimer);
    this.workerIdleTimer = setTimeout(() => {
      if (this.worker && this.waiters.size === 0) {
        this.worker.terminate();
        this.worker = null;
      }
    }, WORKER_IDLE_MS);
  }

  private get idle(): boolean {
    return this.voices.size === 0 && this.wanted.size === 0 && this.pending.size === 0;
  }

  private cancelIdle() {
    if (this.suspendTimer) clearTimeout(this.suspendTimer);
    if (this.trimTimer) clearTimeout(this.trimTimer);
    this.suspendTimer = this.trimTimer = null;
  }

  /**
   * When nothing plays: suspend the AudioContext once fades finish (stops the audio
   * thread from processing silence) and later release the cached buffers.
   */
  private scheduleIdle(afterSec: number) {
    if (!this.idle || !this.ctx) return;
    this.cancelIdle();
    this.suspendTimer = setTimeout(() => {
      if (this.idle && this.ctx && this.ctx.state === 'running') void this.ctx.suspend();
    }, afterSec * 1000 + 300);
    this.trimTimer = setTimeout(() => {
      if (this.idle) this.cache.clear();
    }, TRIM_AFTER_MS);
  }

  private async render(id: string): Promise<[Float32Array, Float32Array]> {
    const worker = this.getWorker();
    if (worker) {
      try {
        return await new Promise((resolve, reject) => {
          const reqId = ++this.reqId;
          this.waiters.set(reqId, { resolve, reject });
          worker.postMessage({ reqId, id, sr: GEN_RATE });
        });
      } catch {
        // fall through to main-thread rendering
      }
    }
    const { renderSound } = await import('./generators');
    return renderSound(id, GEN_RATE);
  }

  private async load(id: string): Promise<AudioBuffer> {
    const hit = this.cache.get(id);
    if (hit) {
      this.cache.delete(id);
      this.cache.set(id, hit); // LRU bump
      return hit;
    }
    const inflight = this.pending.get(id);
    if (inflight) return inflight;
    const ctx = this.ensure();
    const p = (async () => {
      this.onLoading?.(id, true);
      try {
        let buf: AudioBuffer;
        const custom = this.customLoaders.get(id);
        if (custom) {
          buf = await ctx.decodeAudioData(await custom());
        } else {
          const [L, R] = await this.render(id);
          buf = ctx.createBuffer(2, L.length, GEN_RATE);
          buf.copyToChannel(L as Float32Array<ArrayBuffer>, 0);
          buf.copyToChannel(R as Float32Array<ArrayBuffer>, 1);
        }
        this.cache.set(id, buf);
        this.evict();
        return buf;
      } finally {
        this.pending.delete(id);
        this.onLoading?.(id, false);
      }
    })();
    this.pending.set(id, p);
    return p;
  }

  private evict() {
    for (const key of this.cache.keys()) {
      if (this.cache.size <= MAX_CACHED) break;
      if (!this.voices.has(key) && !this.wanted.has(key)) this.cache.delete(key);
    }
  }

  /** Warm the cache without playing (e.g. for the restored mix). */
  preload(ids: string[]) {
    for (const id of ids) void this.load(id).catch(() => {});
  }

  /** Renders the given sounds now (used by the boot splash). Never rejects. */
  async warm(ids: string[]): Promise<void> {
    await Promise.all(ids.map((id) => this.load(id).catch(() => undefined)));
    // Pre-rendering created the AudioContext; put it to sleep until Play (keep the buffers).
    if (this.idle && this.ctx && this.ctx.state === 'running') void this.ctx.suspend();
  }

  async play(id: string, volume: number, pan = 0): Promise<void> {
    this.cancelIdle();
    this.wanted.set(id, { volume, pan });
    if (this.voices.has(id)) {
      this.setVolume(id, volume);
      this.setPan(id, pan);
      return;
    }
    const buf = await this.load(id);
    const want = this.wanted.get(id);
    if (!want || this.voices.has(id)) return; // stopped (or started) meanwhile
    const ctx = this.ensure();
    const src = ctx.createBufferSource();
    src.buffer = buf;
    src.loop = true;
    const p = ctx.createStereoPanner();
    p.pan.value = Math.max(-1, Math.min(1, want.pan));
    const g = ctx.createGain();
    g.gain.value = 0;
    g.gain.setTargetAtTime(Math.max(0, Math.min(1, want.volume)), ctx.currentTime, 0.45);
    src.connect(p);
    p.connect(g);
    g.connect(this.master!);
    // Start at a random offset so repeated plays don't always sound identical
    // (except breathing guides, which must start at the inhale).
    const offset = id.startsWith('r-breath') ? 0 : Math.random() * buf.duration;
    src.start(0, offset);
    this.voices.set(id, { src, gain: g, pan: p, startedAt: ctx.currentTime - offset, duration: buf.duration });
  }

  stop(id: string, fade = 0.8) {
    this.wanted.delete(id);
    const v = this.voices.get(id);
    if (!v || !this.ctx) {
      this.scheduleIdle(0);
      return;
    }
    this.voices.delete(id);
    const now = this.ctx.currentTime;
    v.gain.gain.cancelScheduledValues(now);
    v.gain.gain.setValueAtTime(v.gain.gain.value, now);
    v.gain.gain.setTargetAtTime(0, now, fade / 4);
    try {
      v.src.stop(now + fade + 0.2);
    } catch {
      /* already stopped */
    }
    v.src.onended = () => {
      v.src.disconnect();
      v.pan.disconnect();
      v.gain.disconnect();
    };
    this.scheduleIdle(fade + 0.2);
  }

  stopAll(fade = 0.8) {
    for (const id of [...this.voices.keys(), ...this.wanted.keys()]) this.stop(id, fade);
  }

  setVolume(id: string, volume: number) {
    const w = this.wanted.get(id);
    if (w) w.volume = volume;
    const v = this.voices.get(id);
    if (v && this.ctx) v.gain.gain.setTargetAtTime(Math.max(0, Math.min(1, volume)), this.ctx.currentTime, 0.06);
  }

  setPan(id: string, pan: number) {
    const w = this.wanted.get(id);
    if (w) w.pan = pan;
    const v = this.voices.get(id);
    if (v && this.ctx) v.pan.pan.setTargetAtTime(Math.max(-1, Math.min(1, pan)), this.ctx.currentTime, 0.06);
  }

  setMaster(volume: number) {
    this.masterVolume = Math.max(0, Math.min(1, volume));
    if (this.master && this.ctx) this.master.gain.setTargetAtTime(this.masterVolume, this.ctx.currentTime, 0.08);
  }

  /** Slowly fades the master to silence (sleep timer). Resolves when done. */
  fadeOutMaster(seconds: number): Promise<void> {
    if (!this.master || !this.ctx) return Promise.resolve();
    const now = this.ctx.currentTime;
    this.master.gain.cancelScheduledValues(now);
    this.master.gain.setValueAtTime(this.master.gain.value, now);
    this.master.gain.linearRampToValueAtTime(0, now + seconds);
    return new Promise((r) => setTimeout(r, seconds * 1000));
  }

  restoreMaster() {
    if (!this.master || !this.ctx) return;
    const now = this.ctx.currentTime;
    this.master.gain.cancelScheduledValues(now);
    this.master.gain.setTargetAtTime(this.masterVolume, now, 0.1);
  }

  /** Position (seconds) inside the loop of a playing voice. */
  loopPosition(id: string): number | null {
    const v = this.voices.get(id);
    if (!v || !this.ctx) return null;
    return (((this.ctx.currentTime - v.startedAt) % v.duration) + v.duration) % v.duration;
  }

  isPlaying(id: string): boolean {
    return this.voices.has(id);
  }

  /** Soft bell used for timer phase changes. */
  chime(volume = 0.35) {
    this.cancelIdle();
    const ctx = this.ensure();
    // Let the bell ring out, then go back to sleep if nothing else is playing.
    setTimeout(() => this.scheduleIdle(0), 2500);
    const now = ctx.currentTime;
    const out = ctx.createGain();
    out.gain.value = volume;
    out.connect(ctx.destination);
    for (const [f, a, d] of [
      [880, 0.5, 2.2],
      [880 * 2.76, 0.15, 0.9],
      [1318.5, 0.3, 1.8],
    ] as const) {
      const o = ctx.createOscillator();
      o.frequency.value = f;
      const g = ctx.createGain();
      g.gain.setValueAtTime(0, now);
      g.gain.linearRampToValueAtTime(a, now + 0.005);
      g.gain.exponentialRampToValueAtTime(0.0001, now + d);
      o.connect(g);
      g.connect(out);
      o.start(now);
      o.stop(now + d + 0.05);
    }
  }
}

export const engine = new CalmEngine();
