// Web Audio API ambient synthesizer fallback for browser preview & live playback
class WebAudioEngine {
  private ctx: AudioContext | null = null;
  private masterGain: GainNode | null = null;
  private channels = new Map<string, { gain: GainNode; panner: StereoPannerNode; source: AudioNode }>();
  private masterVol = 0.8;

  private ensureContext(): AudioContext {
    if (!this.ctx) {
      const AudioCtx = window.AudioContext || (window as unknown as { webkitAudioContext: typeof AudioContext }).webkitAudioContext;
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

  private createNoiseBuffer(ctx: AudioContext, type: string): AudioBuffer {
    const bufferSize = ctx.sampleRate * 4; // 4 seconds looping buffer
    const buffer = ctx.createBuffer(1, bufferSize, ctx.sampleRate);
    const data = buffer.getChannelData(0);
    let lastOut = 0.0;

    for (let i = 0; i < bufferSize; i++) {
      const white = Math.random() * 2 - 1;
      if (type === 'ch-noise') {
        // Brown noise
        lastOut = (lastOut + 0.02 * white) / 1.02;
        data[i] = lastOut * 3.5;
      } else if (type === 'ch-rain') {
        lastOut = (lastOut + 0.04 * white) / 1.04;
        const patter = Math.random() > 0.98 ? (Math.random() * 2 - 1) * 0.5 : 0;
        data[i] = lastOut * 2.2 + white * 0.15 + patter;
      } else if (type === 'ch-waves') {
        lastOut = (lastOut + 0.015 * white) / 1.015;
        const t = i / ctx.sampleRate;
        const swell = Math.pow(Math.sin((t * Math.PI * 2) / 4) * 0.5 + 0.5, 2);
        data[i] = lastOut * 3.0 * (0.2 + 0.8 * swell);
      } else if (type === 'ch-wind') {
        lastOut = (lastOut + 0.03 * white) / 1.03;
        const t = i / ctx.sampleRate;
        const gust = Math.sin((t * Math.PI * 2) / 3.5) * 0.4 + 0.6;
        data[i] = (lastOut * 2.0 + white * 0.1) * gust;
      } else if (type === 'ch-fire') {
        lastOut = (lastOut + 0.025 * white) / 1.025;
        const crackle = Math.random() > 0.994 ? (Math.random() * 2 - 1) * 0.9 : 0;
        data[i] = lastOut * 2.5 + crackle;
      } else if (type === 'ch-stream') {
        lastOut = (lastOut + 0.06 * white) / 1.06;
        const t = i / ctx.sampleRate;
        const bubble = Math.sin(t * 28) * 0.2 + 0.8;
        data[i] = (lastOut * 1.5 + white * 0.25) * bubble;
      } else {
        lastOut = (lastOut + 0.03 * white) / 1.03;
        data[i] = lastOut * 2.5;
      }
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
        // Binaural 210Hz
        const osc = ctx.createOscillator();
        osc.type = 'sine';
        osc.frequency.setValueAtTime(210, ctx.currentTime);
        osc.start();
        sourceNode = osc;
      } else if (channelId === 'ch-birds') {
        // Periodic gentle chirp modulated sine + low rustle
        const osc = ctx.createOscillator();
        osc.type = 'triangle';
        osc.frequency.setValueAtTime(2600, ctx.currentTime);
        const lfo = ctx.createOscillator();
        lfo.frequency.setValueAtTime(3, ctx.currentTime);
        const lfoGain = ctx.createGain();
        lfoGain.gain.setValueAtTime(400, ctx.currentTime);
        lfo.connect(lfoGain);
        lfoGain.connect(osc.frequency);
        osc.start();
        lfo.start();
        sourceNode = osc;
      } else {
        const buf = this.createNoiseBuffer(ctx, channelId);
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
