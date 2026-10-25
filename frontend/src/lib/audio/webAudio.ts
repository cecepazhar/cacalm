// Web Audio API ambient synthesizer fallback for browser preview & live playback
class WebAudioEngine {
  private ctx: AudioContext | null = null;
  private masterGain: GainNode | null = null;
  private channels = new Map<string, { gain: GainNode; panner: StereoPannerNode; source: AudioNode }>();
  private masterVol = 0.8;

  private ensureContext(): AudioContext {
    if (!this.ctx) {
      const AudioCtx =
        window.AudioContext ||
        (window as unknown as { webkitAudioContext: typeof AudioContext }).webkitAudioContext;
      this.ctx = new AudioCtx();
      this.masterGain = this.ctx.createGain();
      this.masterGain.gain.setValueAtTime(this.masterVol, this.ctx.currentTime);
      this.masterGain.connect(this.ctx.destination);
    }
    if (this.ctx.state === 'suspended') {
      void this.ctx.resume();
    }
    return this.ctx;
  }

  private createSeamlessNoiseBuffer(ctx: AudioContext, type: string): AudioBuffer {
    // 16 seconds loop buffer with 1.5s seamless circular crossfade
    const duration = 16.0;
    const fadeDuration = 1.5;
    const sampleRate = ctx.sampleRate;
    const bodySamples = Math.floor(sampleRate * duration);
    const fadeSamples = Math.floor(sampleRate * fadeDuration);
    const totalSamples = bodySamples + fadeSamples;

    const raw = new Float32Array(totalSamples);
    let lastBrown = 0.0;
    let b0 = 0.0, b1 = 0.0, b2 = 0.0, b3 = 0.0, b4 = 0.0, b5 = 0.0, b6 = 0.0;

    for (let i = 0; i < totalSamples; i++) {
      const white = Math.random() * 2 - 1;
      const t = i / sampleRate;

      // Pink noise approximation
      b0 = 0.99886 * b0 + white * 0.0555179;
      b1 = 0.99332 * b1 + white * 0.0750759;
      b2 = 0.969 * b2 + white * 0.153852;
      b3 = 0.8665 * b3 + white * 0.3104856;
      b4 = 0.55 * b4 + white * 0.5329522;
      b5 = -0.7616 * b5 - white * 0.016898;
      const pink = (b0 + b1 + b2 + b3 + b4 + b5 + b6 + white * 0.5362) * 0.11;
      b6 = white * 0.115926;

      // Brown noise approximation
      lastBrown = (lastBrown + 0.02 * white) / 1.02;
      const brown = lastBrown * 3.2;

      let s = 0.0;
      if (type === 'ch-noise') {
        s = brown * 0.7;
      } else if (type === 'ch-rain') {
        const drop = Math.random() > 0.982 ? (Math.random() * 2 - 1) * 0.45 : 0;
        s = brown * 0.35 + pink * 0.4 + drop;
      } else if (type === 'ch-waves') {
        const swell = Math.pow(Math.sin((t * Math.PI * 2) / 8.0) * 0.5 + 0.5, 2.0);
        s = (brown * 0.5 + pink * 0.3) * (0.2 + 0.8 * swell);
      } else if (type === 'ch-wind') {
        const gust = Math.sin((t * Math.PI * 2) / 5.2) * 0.35 + 0.65;
        s = (brown * 0.4 + pink * 0.2) * gust;
      } else if (type === 'ch-fire') {
        const pop = Math.random() > 0.992 ? (Math.random() * 2 - 1) * 0.8 : 0;
        s = brown * 0.4 + pop;
      } else if (type === 'ch-stream') {
        const bubble = Math.sin(t * 16) * 0.25 + 0.75;
        s = (pink * 0.4 + white * 0.1) * bubble;
      } else if (type === 'ch-cafe') {
        const murmur = Math.sin(t * Math.PI * 2 * 180) * 0.06 + Math.sin(t * Math.PI * 2 * 260) * 0.04;
        const clink = Math.random() > 0.998 ? (Math.random() * 2 - 1) * 0.5 : 0;
        s = brown * 0.35 + pink * 0.25 + murmur + clink;
      } else if (type === 'ch-spatial') {
        const lfo = Math.sin(t * Math.PI * 2 * 0.15);
        const tone = Math.sin(t * Math.PI * 2 * (320 + 35 * lfo)) * 0.2;
        s = brown * 0.25 + tone;
      } else {
        s = brown * 0.4;
      }

      raw[i] = Math.max(-1.0, Math.min(1.0, s));
    }

    // Circular equal-power crossfade between the tail and head
    for (let i = 0; i < fadeSamples; i++) {
      const norm = i / fadeSamples;
      // Equal power curve
      const gainHead = Math.sin(norm * 0.5 * Math.PI);
      const gainTail = Math.cos(norm * 0.5 * Math.PI);
      const tailVal = raw[bodySamples + i];
      const headVal = raw[i];
      raw[i] = gainTail * tailVal + gainHead * headVal;
    }

    const buffer = ctx.createBuffer(1, bodySamples, sampleRate);
    const data = buffer.getChannelData(0);
    for (let i = 0; i < bodySamples; i++) {
      data[i] = raw[i];
    }

    return buffer;
  }

  playChannel(channelId: string, volume: number, pan: number = 0) {
    try {
      const ctx = this.ensureContext();
      this.stopChannel(channelId);

      const gain = ctx.createGain();
      gain.gain.setValueAtTime(Math.max(0, Math.min(1, volume)), ctx.currentTime);

      const panner = ctx.createStereoPanner();
      panner.pan.setValueAtTime(Math.max(-1, Math.min(1, pan)), ctx.currentTime);

      let sourceNode: AudioNode;

      if (channelId === 'ch-binaural') {
        const osc = ctx.createOscillator();
        osc.type = 'sine';
        osc.frequency.setValueAtTime(210, ctx.currentTime);
        osc.start();
        sourceNode = osc;
      } else if (channelId === 'ch-birds') {
        // Continuous synth for birds
        const osc = ctx.createOscillator();
        osc.type = 'triangle';
        osc.frequency.setValueAtTime(2400, ctx.currentTime);
        const lfo = ctx.createOscillator();
        lfo.frequency.setValueAtTime(0.4, ctx.currentTime);
        const lfoGain = ctx.createGain();
        lfoGain.gain.setValueAtTime(600, ctx.currentTime);
        lfo.connect(lfoGain);
        lfoGain.connect(osc.frequency);
        osc.start();
        lfo.start();
        sourceNode = osc;
      } else {
        const buf = this.createSeamlessNoiseBuffer(ctx, channelId);
        const bs = ctx.createBufferSource();
        bs.buffer = buf;
        bs.loop = true;
        bs.start();
        sourceNode = bs;
      }

      sourceNode.connect(panner);
      panner.connect(gain);
      gain.connect(this.masterGain!);

      this.channels.set(channelId, { gain, panner, source: sourceNode });
    } catch {
      // Ignore in non-browser or audio-blocked
    }
  }

  stopChannel(channelId: string) {
    const ch = this.channels.get(channelId);
    if (ch) {
      try {
        if ('stop' in ch.source && typeof (ch.source as { stop: () => void }).stop === 'function') {
          (ch.source as { stop: () => void }).stop();
        }
        ch.source.disconnect();
        ch.panner.disconnect();
        ch.gain.disconnect();
      } catch {
        // Ignore
      }
      this.channels.delete(channelId);
    }
  }

  setChannelVolume(channelId: string, volume: number) {
    const ch = this.channels.get(channelId);
    if (ch && this.ctx) {
      try {
        ch.gain.gain.setValueAtTime(Math.max(0, Math.min(1, volume)), this.ctx.currentTime);
      } catch {
        // Ignore
      }
    }
  }

  setChannelPan(channelId: string, pan: number) {
    const ch = this.channels.get(channelId);
    if (ch && this.ctx) {
      try {
        ch.panner.pan.setValueAtTime(Math.max(-1, Math.min(1, pan)), this.ctx.currentTime);
      } catch {
        // Ignore
      }
    }
  }

  setMasterVolume(volume: number) {
    this.masterVol = Math.max(0, Math.min(1, volume));
    if (this.masterGain && this.ctx) {
      try {
        this.masterGain.gain.setValueAtTime(this.masterVol, this.ctx.currentTime);
      } catch {
        // Ignore
      }
    }
  }
}

export const webAudio = new WebAudioEngine();
