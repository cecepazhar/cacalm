//! Procedural Ambient & Noise Sound Generator.
//!
//! Generates synthetic procedural audio buffers and encodes them to standard
//! 16-bit PCM WAV files with seamless circular crossfade loops.
//!
//! Default sound channels supported:
//! - Rain (filtered noise bed + random droplet impulses)
//! - Ocean Waves (slow LFO-modulated pink/brown noise surge & surf)
//! - Forest Birds (gentle breeze rustle + FM-synthesized bird chirps)
//! - Coffee Shop (muffled murmur + subtle ceramic clinks)
//! - Fireplace (low warm rumble + Poisson wood crackles & pops)
//! - Wind (resonant band-pass filtered noise sweep)
//! - Stream (babbling brook water texture + resonant bubbles)
//! - Brown Noise (integrated white noise 1/f^2)

use std::f32::consts::PI;
use std::fs;
use std::path::{Path, PathBuf};

/// Simple, deterministic, zero-dependency PRNG (Xorshift64*).
#[derive(Clone, Debug)]
pub struct SynthRng {
    state: u64,
}

impl SynthRng {
    pub fn new(seed: u64) -> Self {
        Self {
            state: if seed == 0 {
                0x853c_49e6_748f_ea9b
            } else {
                seed
            },
        }
    }

    #[inline]
    pub fn next_u32(&mut self) -> u32 {
        self.state ^= self.state >> 12;
        self.state ^= self.state << 25;
        self.state ^= self.state >> 27;
        (self.state.wrapping_mul(0x2545_f491_4f6c_dd1d) >> 32) as u32
    }

    #[inline]
    pub fn next_f32(&mut self) -> f32 {
        // Range [-1.0, 1.0]
        (self.next_u32() as f32 / 2_147_483_648.0) - 1.0
    }

    #[inline]
    pub fn next_unit(&mut self) -> f32 {
        // Range [0.0, 1.0]
        self.next_u32() as f32 / 4_294_967_295.0
    }
}

/// One-pole low-pass filter.
#[derive(Clone, Debug, Default)]
pub struct LowPass {
    last: f32,
}

impl LowPass {
    pub fn new() -> Self {
        Self { last: 0.0 }
    }

    #[inline]
    pub fn process(&mut self, sample: f32, cutoff_hz: f32, sample_rate: f32) -> f32 {
        let rc = 1.0 / (2.0 * PI * cutoff_hz.max(10.0));
        let dt = 1.0 / sample_rate;
        let alpha = (dt / (rc + dt)).clamp(0.0001, 1.0);
        self.last += alpha * (sample - self.last);
        self.last
    }
}

/// One-pole high-pass filter.
#[derive(Clone, Debug, Default)]
pub struct HighPass {
    last_in: f32,
    last_out: f32,
}

impl HighPass {
    pub fn new() -> Self {
        Self {
            last_in: 0.0,
            last_out: 0.0,
        }
    }

    #[inline]
    pub fn process(&mut self, sample: f32, cutoff_hz: f32, sample_rate: f32) -> f32 {
        let rc = 1.0 / (2.0 * PI * cutoff_hz.max(10.0));
        let dt = 1.0 / sample_rate;
        let alpha = (rc / (rc + dt)).clamp(0.0001, 1.0);
        self.last_out = alpha * (self.last_out + sample - self.last_in);
        self.last_in = sample;
        self.last_out
    }
}

/// Second-order resonant band-pass filter.
#[derive(Clone, Debug)]
pub struct BandPass {
    x1: f32,
    x2: f32,
    y1: f32,
    y2: f32,
}

impl BandPass {
    pub fn new() -> Self {
        Self {
            x1: 0.0,
            x2: 0.0,
            y1: 0.0,
            y2: 0.0,
        }
    }

    #[inline]
    pub fn process(&mut self, sample: f32, center_hz: f32, q: f32, sample_rate: f32) -> f32 {
        let safe_center = center_hz.clamp(20.0, sample_rate * 0.48);
        let safe_q = q.max(0.1);
        let w0 = 2.0 * PI * (safe_center / sample_rate);
        let alpha = w0.sin() / (2.0 * safe_q);

        let b0 = alpha;
        let b1 = 0.0;
        let b2 = -alpha;
        let a0 = 1.0 + alpha;
        let a1 = -2.0 * w0.cos();
        let a2 = 1.0 - alpha;

        let out = (b0 / a0) * sample + (b1 / a0) * self.x1 + (b2 / a0) * self.x2
            - (a1 / a0) * self.y1
            - (a2 / a0) * self.y2;

        self.x2 = self.x1;
        self.x1 = sample;
        self.y2 = self.y1;
        self.y1 = out;

        out
    }
}

impl Default for BandPass {
    fn default() -> Self {
        Self::new()
    }
}

/// Pink noise generator using Paul Kellet's filtered noise approximation.
#[derive(Clone, Debug, Default)]
pub struct PinkNoise {
    b0: f32,
    b1: f32,
    b2: f32,
    b3: f32,
    b4: f32,
    b5: f32,
    b6: f32,
}

impl PinkNoise {
    pub fn new() -> Self {
        Self::default()
    }

    #[inline]
    pub fn next_sample(&mut self, white: f32) -> f32 {
        self.b0 = 0.99886 * self.b0 + white * 0.055_517_9;
        self.b1 = 0.99332 * self.b1 + white * 0.075_075_9;
        self.b2 = 0.96900 * self.b2 + white * 0.153_852;
        self.b3 = 0.86650 * self.b3 + white * 0.310_485_6;
        self.b4 = 0.55000 * self.b4 + white * 0.532_952_2;
        self.b5 = -0.7616 * self.b5 - white * 0.016_898;
        let pink = self.b0
            + self.b1
            + self.b2
            + self.b3
            + self.b4
            + self.b5
            + self.b6
            + white * 0.5362;
        self.b6 = white * 0.115_926;
        pink * 0.12
    }
}

/// Brown noise (Brownian motion / 1/f^2) generator.
#[derive(Clone, Debug, Default)]
pub struct BrownNoise {
    last: f32,
}

impl BrownNoise {
    pub fn new() -> Self {
        Self::default()
    }

    #[inline]
    pub fn next_sample(&mut self, white: f32) -> f32 {
        self.last = (self.last + (0.025 * white)) / 1.02;
        self.last * 3.2
    }
}

/// Performs circular seamless crossfading at loop boundaries.
///
/// Blends the extra tail of length `fade_samples` into the beginning of `samples`
/// so that the transition from `samples.last()` back to `samples[0]` is smooth.
pub fn apply_circular_crossfade(samples: &mut [f32], total_len: usize, fade_samples: usize) {
    if samples.len() < total_len + fade_samples || fade_samples == 0 {
        return;
    }
    for i in 0..fade_samples {
        let t = i as f32 / fade_samples as f32;
        let tail_val = samples[total_len + i];
        let head_val = samples[i];
        samples[i] = (1.0 - t) * tail_val + t * head_val;
    }
}

// ---------------------------------------------------------------------------
// Individual Ambient Sound Synthesis Functions
// ---------------------------------------------------------------------------

pub fn generate_brown_noise(duration_secs: f32, sample_rate: u32) -> Vec<f32> {
    let mut rng = SynthRng::new(42);
    let mut brown = BrownNoise::new();
    let body_samples = (sample_rate as f32 * duration_secs) as usize;
    let fade_samples = (sample_rate as f32 * 0.4) as usize;
    let total = body_samples + fade_samples;

    let mut buf = Vec::with_capacity(total);
    for _ in 0..total {
        let w = rng.next_f32();
        buf.push(brown.next_sample(w).clamp(-1.0, 1.0));
    }
    apply_circular_crossfade(&mut buf, body_samples, fade_samples);
    buf.truncate(body_samples);
    buf
}

pub fn generate_rain(duration_secs: f32, sample_rate: u32) -> Vec<f32> {
    let mut rng = SynthRng::new(101);
    let mut pink = PinkNoise::new();
    let mut lp = LowPass::new();
    let mut hp = HighPass::new();
    let mut drop_bp = BandPass::new();

    let body_samples = (sample_rate as f32 * duration_secs) as usize;
    let fade_samples = (sample_rate as f32 * 0.4) as usize;
    let total = body_samples + fade_samples;

    let mut drop_env = 0.0f32;
    let mut drop_freq = 2400.0f32;
    let mut buf = Vec::with_capacity(total);

    for _ in 0..total {
        let white = rng.next_f32();
        let p = pink.next_sample(white);

        // Bed of steady rain noise
        let filtered_bed = lp.process(p, 3200.0, sample_rate as f32);
        let rain_bed = hp.process(filtered_bed, 180.0, sample_rate as f32) * 0.55;

        // Occasional raindrop impact
        if rng.next_unit() < (85.0 / sample_rate as f32) {
            drop_env = 0.45 * rng.next_unit() + 0.15;
            drop_freq = 1600.0 + rng.next_unit() * 1800.0;
        }

        let drop_sound = drop_bp.process(white * drop_env, drop_freq, 6.0, sample_rate as f32);
        drop_env *= 0.992; // rapid decay

        let mixed = rain_bed + drop_sound * 0.4;
        buf.push(mixed.clamp(-1.0, 1.0));
    }

    apply_circular_crossfade(&mut buf, body_samples, fade_samples);
    buf.truncate(body_samples);
    buf
}

pub fn generate_ocean_waves(duration_secs: f32, sample_rate: u32) -> Vec<f32> {
    let mut rng = SynthRng::new(202);
    let mut pink = PinkNoise::new();
    let mut brown = BrownNoise::new();
    let mut lp = LowPass::new();

    let body_samples = (sample_rate as f32 * duration_secs) as usize;
    let fade_samples = (sample_rate as f32 * 0.5) as usize;
    let total = body_samples + fade_samples;

    // LFO frequency matches duration to make the swell cycle cleanly
    let lfo_freq = 1.0 / duration_secs;
    let mut buf = Vec::with_capacity(total);

    for i in 0..total {
        let t = i as f32 / sample_rate as f32;
        let white = rng.next_f32();
        let p = pink.next_sample(white);
        let b = brown.next_sample(white);
        let base = p * 0.6 + b * 0.4;

        // Wave swell cycle [0.0, 1.0]
        let wave_phase = (2.0 * PI * lfo_freq * t).sin() * 0.5 + 0.5;
        // Asymmetric wave shape (sharper crest, softer trough)
        let wave_shape = wave_phase.powf(1.6);

        let cutoff = 250.0 + wave_shape * 1900.0;
        let filtered = lp.process(base, cutoff, sample_rate as f32);
        let gain = 0.18 + wave_shape * 0.62;

        buf.push((filtered * gain).clamp(-1.0, 1.0));
    }

    apply_circular_crossfade(&mut buf, body_samples, fade_samples);
    buf.truncate(body_samples);
    buf
}

pub fn generate_birds(duration_secs: f32, sample_rate: u32) -> Vec<f32> {
    let mut rng = SynthRng::new(303);
    let mut pink = PinkNoise::new();
    let mut lp = LowPass::new();

    let body_samples = (sample_rate as f32 * duration_secs) as usize;
    let fade_samples = (sample_rate as f32 * 0.4) as usize;
    let total = body_samples + fade_samples;

    let mut buf = Vec::with_capacity(total);

    // Forest ambient foliage bed + periodic bird song bursts
    let mut chirp_timer = 0;
    let mut chirp_duration = 0;
    let mut chirp_carrier = 3200.0f32;
    let mut chirp_mod_depth = 400.0f32;
    let mut chirp_phase = 0.0f32;

    for _ in 0..total {
        let white = rng.next_f32();
        let foliage = lp.process(pink.next_sample(white), 600.0, sample_rate as f32) * 0.22;

        chirp_timer += 1;
        if chirp_timer > (sample_rate as f32 * 1.8) as usize && rng.next_unit() < 0.0008 {
            chirp_timer = 0;
            chirp_duration = ((0.12 + rng.next_unit() * 0.18) * sample_rate as f32) as usize;
            chirp_carrier = 2800.0 + rng.next_unit() * 1200.0;
            chirp_mod_depth = 250.0 + rng.next_unit() * 300.0;
        }

        let mut bird_sample = 0.0;
        if chirp_duration > 0 {
            chirp_duration -= 1;
            let env = (chirp_duration as f32 / (sample_rate as f32 * 0.25)).clamp(0.0, 1.0);
            let fm = (chirp_phase * 18.0).sin() * chirp_mod_depth;
            let freq = (chirp_carrier + fm).max(100.0);
            chirp_phase += 2.0 * PI * (freq / sample_rate as f32);
            bird_sample = chirp_phase.sin() * env * 0.35;
        }

        buf.push((foliage + bird_sample).clamp(-1.0, 1.0));
    }

    apply_circular_crossfade(&mut buf, body_samples, fade_samples);
    buf.truncate(body_samples);
    buf
}

pub fn generate_cafe(duration_secs: f32, sample_rate: u32) -> Vec<f32> {
    let mut rng = SynthRng::new(404);
    let mut pink = PinkNoise::new();
    let mut bp1 = BandPass::new();
    let mut bp2 = BandPass::new();
    let mut clink_bp = BandPass::new();

    let body_samples = (sample_rate as f32 * duration_secs) as usize;
    let fade_samples = (sample_rate as f32 * 0.4) as usize;
    let total = body_samples + fade_samples;

    let mut clink_env = 0.0f32;
    let mut clink_freq = 2200.0f32;
    let mut buf = Vec::with_capacity(total);

    for _ in 0..total {
        let white = rng.next_f32();
        let p = pink.next_sample(white);

        // Vocal formant-like filtered noise mimicking distant chatter
        let murmur1 = bp1.process(p, 420.0, 2.5, sample_rate as f32) * 0.35;
        let murmur2 = bp2.process(p, 850.0, 3.0, sample_rate as f32) * 0.25;

        // Occasional ceramic cup / spoon clink
        if rng.next_unit() < (2.0 / sample_rate as f32) {
            clink_env = 0.45;
            clink_freq = if rng.next_unit() > 0.5 {
                2400.0
            } else {
                3100.0
            };
        }

        let clink = clink_bp.process(white * clink_env, clink_freq, 15.0, sample_rate as f32);
        clink_env *= 0.996;

        let mixed = murmur1 + murmur2 + clink * 0.35;
        buf.push(mixed.clamp(-1.0, 1.0));
    }

    apply_circular_crossfade(&mut buf, body_samples, fade_samples);
    buf.truncate(body_samples);
    buf
}

pub fn generate_fire(duration_secs: f32, sample_rate: u32) -> Vec<f32> {
    let mut rng = SynthRng::new(505);
    let mut brown = BrownNoise::new();
    let mut lp = LowPass::new();
    let mut crackle_bp = BandPass::new();

    let body_samples = (sample_rate as f32 * duration_secs) as usize;
    let fade_samples = (sample_rate as f32 * 0.4) as usize;
    let total = body_samples + fade_samples;

    let mut crackle_env = 0.0f32;
    let mut crackle_freq = 1200.0f32;
    let mut buf = Vec::with_capacity(total);

    for _ in 0..total {
        let white = rng.next_f32();
        let b = brown.next_sample(white);
        let warmth = lp.process(b, 160.0, sample_rate as f32) * 0.45;

        // Wood crackle impulses (Poisson arrival)
        if rng.next_unit() < (38.0 / sample_rate as f32) {
            crackle_env = 0.35 + rng.next_unit() * 0.55;
            crackle_freq = 900.0 + rng.next_unit() * 2200.0;
        }

        let crackle = crackle_bp.process(white * crackle_env, crackle_freq, 7.0, sample_rate as f32);
        crackle_env *= 0.985; // very sharp pop

        let mixed = warmth + crackle * 0.5;
        buf.push(mixed.clamp(-1.0, 1.0));
    }

    apply_circular_crossfade(&mut buf, body_samples, fade_samples);
    buf.truncate(body_samples);
    buf
}

pub fn generate_wind(duration_secs: f32, sample_rate: u32) -> Vec<f32> {
    let mut rng = SynthRng::new(606);
    let mut pink = PinkNoise::new();
    let mut bp = BandPass::new();

    let body_samples = (sample_rate as f32 * duration_secs) as usize;
    let fade_samples = (sample_rate as f32 * 0.4) as usize;
    let total = body_samples + fade_samples;

    let mut buf = Vec::with_capacity(total);

    for i in 0..total {
        let t = i as f32 / sample_rate as f32;
        let white = rng.next_f32();
        let p = pink.next_sample(white);

        // Undulating gust frequency
        let gust1 = (2.0 * PI * 0.11 * t).sin() * 140.0;
        let gust2 = (2.0 * PI * 0.23 * t + 1.2).sin() * 90.0;
        let center_hz = 320.0 + gust1 + gust2;

        let wind = bp.process(p, center_hz, 3.2, sample_rate as f32);
        let amp = 0.35 + ((2.0 * PI * 0.11 * t).sin() * 0.5 + 0.5) * 0.35;

        buf.push((wind * amp).clamp(-1.0, 1.0));
    }

    apply_circular_crossfade(&mut buf, body_samples, fade_samples);
    buf.truncate(body_samples);
    buf
}

pub fn generate_stream(duration_secs: f32, sample_rate: u32) -> Vec<f32> {
    let mut rng = SynthRng::new(707);
    let mut pink = PinkNoise::new();
    let mut lp = LowPass::new();
    let mut bp1 = BandPass::new();
    let mut bp2 = BandPass::new();

    let body_samples = (sample_rate as f32 * duration_secs) as usize;
    let fade_samples = (sample_rate as f32 * 0.4) as usize;
    let total = body_samples + fade_samples;

    let mut buf = Vec::with_capacity(total);

    for i in 0..total {
        let t = i as f32 / sample_rate as f32;
        let white = rng.next_f32();
        let p = pink.next_sample(white);

        // Steady rushing water
        let flow = lp.process(p, 1400.0, sample_rate as f32) * 0.38;

        // Bubbling modulation
        let f1 = 520.0 + (2.0 * PI * 1.8 * t).sin() * 120.0;
        let f2 = 980.0 + (2.0 * PI * 2.7 * t + 0.8).sin() * 180.0;

        let bubble1 = bp1.process(p, f1, 4.0, sample_rate as f32) * 0.22;
        let bubble2 = bp2.process(p, f2, 5.0, sample_rate as f32) * 0.18;

        let mixed = flow + bubble1 + bubble2;
        buf.push(mixed.clamp(-1.0, 1.0));
    }

    apply_circular_crossfade(&mut buf, body_samples, fade_samples);
    buf.truncate(body_samples);
    buf
}

// ---------------------------------------------------------------------------
// WAV Encoding & Default Sound Registry
// ---------------------------------------------------------------------------

/// Encodes raw f32 audio samples into standard 16-bit PCM RIFF WAV format.
pub fn encode_wav(samples: &[f32], sample_rate: u32, channels: u16) -> Vec<u8> {
    let bits_per_sample: u16 = 16;
    let bytes_per_sample: u16 = bits_per_sample / 8;
    let block_align: u16 = channels * bytes_per_sample;
    let byte_rate: u32 = sample_rate * block_align as u32;
    let data_size: u32 = samples.len() as u32 * bytes_per_sample as u32;
    let file_size: u32 = 36 + data_size;

    let mut buf = Vec::with_capacity(44 + data_size as usize);
    // RIFF header
    buf.extend_from_slice(b"RIFF");
    buf.extend_from_slice(&file_size.to_le_bytes());
    buf.extend_from_slice(b"WAVE");

    // "fmt " chunk
    buf.extend_from_slice(b"fmt ");
    buf.extend_from_slice(&16u32.to_le_bytes()); // subchunk size = 16 for PCM
    buf.extend_from_slice(&1u16.to_le_bytes());  // AudioFormat = 1 (PCM)
    buf.extend_from_slice(&channels.to_le_bytes());
    buf.extend_from_slice(&sample_rate.to_le_bytes());
    buf.extend_from_slice(&byte_rate.to_le_bytes());
    buf.extend_from_slice(&block_align.to_le_bytes());
    buf.extend_from_slice(&bits_per_sample.to_le_bytes());

    // "data" chunk
    buf.extend_from_slice(b"data");
    buf.extend_from_slice(&data_size.to_le_bytes());

    for &s in samples {
        let val = (s.clamp(-1.0, 1.0) * 32_767.0) as i16;
        buf.extend_from_slice(&val.to_le_bytes());
    }

    buf
}

/// Metadata description for built-in default sounds.
#[derive(Debug, Clone, Copy)]
pub struct SoundDef {
    pub id: &'static str,
    pub name: &'static str,
    pub file_name: &'static str,
    pub category: &'static str,
    pub duration_secs: f32,
}

pub const DEFAULT_SOUNDS: &[SoundDef] = &[
    SoundDef {
        id: "ch-rain",
        name: "Rain",
        file_name: "rain.wav",
        category: "Nature",
        duration_secs: 6.0,
    },
    SoundDef {
        id: "ch-waves",
        name: "Ocean Waves",
        file_name: "waves.wav",
        category: "Nature",
        duration_secs: 8.0,
    },
    SoundDef {
        id: "ch-birds",
        name: "Forest Birds",
        file_name: "birds.wav",
        category: "Nature",
        duration_secs: 7.0,
    },
    SoundDef {
        id: "ch-cafe",
        name: "Coffee Shop",
        file_name: "cafe.wav",
        category: "Ambience",
        duration_secs: 6.0,
    },
    SoundDef {
        id: "ch-fire",
        name: "Fireplace",
        file_name: "fire.wav",
        category: "Ambience",
        duration_secs: 6.0,
    },
    SoundDef {
        id: "ch-wind",
        name: "Wind",
        file_name: "wind.wav",
        category: "Nature",
        duration_secs: 7.0,
    },
    SoundDef {
        id: "ch-stream",
        name: "Stream",
        file_name: "stream.wav",
        category: "Nature",
        duration_secs: 6.0,
    },
    SoundDef {
        id: "ch-noise",
        name: "Brown Noise",
        file_name: "noise.wav",
        category: "Noise",
        duration_secs: 6.0,
    },
];

/// Synthesize a sound by definition.
pub fn synthesize_sound(def: &SoundDef, sample_rate: u32) -> Vec<f32> {
    match def.id {
        "ch-rain" => generate_rain(def.duration_secs, sample_rate),
        "ch-waves" => generate_ocean_waves(def.duration_secs, sample_rate),
        "ch-birds" => generate_birds(def.duration_secs, sample_rate),
        "ch-cafe" => generate_cafe(def.duration_secs, sample_rate),
        "ch-fire" => generate_fire(def.duration_secs, sample_rate),
        "ch-wind" => generate_wind(def.duration_secs, sample_rate),
        "ch-stream" => generate_stream(def.duration_secs, sample_rate),
        "ch-noise" => generate_brown_noise(def.duration_secs, sample_rate),
        _ => generate_brown_noise(def.duration_secs, sample_rate),
    }
}

/// Ensures all default sounds exist in `dir`, synthesizing them if missing.
///
/// Returns the list of created or verified file paths.
pub fn ensure_default_sounds(dir: &Path) -> Result<Vec<(String, PathBuf)>, std::io::Error> {
    fs::create_dir_all(dir)?;
    let sample_rate = 44_100;
    let mut out = Vec::new();

    for def in DEFAULT_SOUNDS {
        let file_path = dir.join(def.file_name);
        let needs_write = match fs::metadata(&file_path) {
            Ok(meta) => meta.len() == 0,
            Err(_) => true,
        };

        if needs_write {
            let samples = synthesize_sound(def, sample_rate);
            let wav_bytes = encode_wav(&samples, sample_rate, 1);
            fs::write(&file_path, wav_bytes)?;
        }

        out.push((def.id.to_string(), file_path));
    }

    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rng_produces_values_in_unit_range() {
        let mut rng = SynthRng::new(12345);
        for _ in 0..100 {
            let v = rng.next_f32();
            assert!((-1.0..=1.0).contains(&v));
            let u = rng.next_unit();
            assert!((0.0..=1.0).contains(&u));
        }
    }

    #[test]
    fn brown_noise_stays_within_bounds() {
        let samples = generate_brown_noise(0.5, 44100);
        assert_eq!(samples.len(), 22050);
        for &s in &samples {
            assert!((-1.0..=1.0).contains(&s));
        }
    }

    #[test]
    fn wav_encoder_creates_valid_riff_header() {
        let samples = vec![0.0, 0.5, -0.5, 0.25];
        let bytes = encode_wav(&samples, 44100, 1);
        assert_eq!(&bytes[0..4], b"RIFF");
        assert_eq!(&bytes[8..12], b"WAVE");
        assert_eq!(&bytes[12..16], b"fmt ");
        assert_eq!(&bytes[36..40], b"data");
        assert_eq!(bytes.len(), 44 + samples.len() * 2);
    }

    #[test]
    fn ensure_default_sounds_creates_all_eight_files() {
        let tmp = std::env::temp_dir().join(format!("cacalm-synth-test-{}", uuid::Uuid::now_v7()));
        let results = ensure_default_sounds(&tmp).expect("should generate default sounds");
        assert_eq!(results.len(), 8);
        for (id, path) in results {
            assert!(path.exists(), "path for {id} must exist: {path:?}");
            let meta = fs::metadata(&path).unwrap();
            assert!(meta.len() > 1000, "audio file {path:?} should not be empty");
        }
        let _ = fs::remove_dir_all(&tmp);
    }
}
