use parking_lot::Mutex;
use rodio::{Decoder, OutputStream, OutputStreamHandle, Source, SpatialSink};
use std::collections::HashMap;
use std::fs::File;
use std::io::BufReader;
use std::sync::mpsc;
use std::thread;
use uuid::Uuid;

/// Opaque channel handle.
#[derive(Debug, Clone, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub struct ChannelId(pub String);

impl ChannelId {
    pub fn new() -> Self {
        Self(Uuid::now_v7().to_string())
    }

    pub fn from_string(s: impl Into<String>) -> Self {
        Self(s.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl Default for ChannelId {
    fn default() -> Self {
        Self::new()
    }
}

impl From<String> for ChannelId {
    fn from(s: String) -> Self {
        Self(s)
    }
}

impl From<&str> for ChannelId {
    fn from(s: &str) -> Self {
        Self(s.to_string())
    }
}

impl std::fmt::Display for ChannelId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Per-channel configuration supplied at open time.
#[derive(Debug, Clone)]
pub struct ChannelConfig {
    pub channel: ChannelId,
    pub name: String,
    pub source_path: String,
    pub loop_enabled: bool,
}

struct ChannelState {
    sink: Option<SpatialSink>,
    config: ChannelConfig,
    gain: f32,
    muted: bool,
    pan: f32,
}

pub const MAX_CHANNELS: usize = 8;
pub const DEFAULT_GAIN: f32 = 0.8;
pub const SPATIAL_FACTOR: f32 = 2.0;

#[derive(Debug, thiserror::Error)]
pub enum AudioError {
    #[error("no audio output device is available: {0}")]
    NoDevice(String),
    #[error("cannot open file: {0}")]
    Open(String),
    #[error("unsupported or corrupt audio: {0}")]
    Decode(String),
}

/// Multi-channel playback engine with spatial audio panning.
///
/// `rodio::OutputStream` is `!Send + !Sync` on Linux/ALSA, so it is owned by
/// a dedicated worker thread. The `OutputStreamHandle` is `Send + Sync` and
/// safely shared.
pub struct AudioEngine {
    handle: OutputStreamHandle,
    _drop_tx: mpsc::Sender<()>,
    _thread: Option<thread::JoinHandle<()>>,
    channels: Mutex<HashMap<ChannelId, ChannelState>>,
    master_volume: Mutex<f32>,
}

impl AudioEngine {
    pub fn new() -> Result<Self, AudioError> {
        let (init_tx, init_rx) = mpsc::sync_channel::<Result<OutputStreamHandle, String>>(1);
        let (drop_tx, drop_rx) = mpsc::channel::<()>();

        let join_handle = thread::Builder::new()
            .name("cacalm-audio-stream".into())
            .spawn(move || match OutputStream::try_default() {
                Ok((stream, handle)) => {
                    let _ = init_tx.send(Ok(handle));
                    let _ = drop_rx.recv();
                    drop(stream);
                }
                Err(e) => {
                    let _ = init_tx.send(Err(e.to_string()));
                }
            })
            .map_err(|e| AudioError::NoDevice(e.to_string()))?;

        let handle = init_rx
            .recv()
            .map_err(|e| AudioError::NoDevice(e.to_string()))?
            .map_err(AudioError::NoDevice)?;

        Ok(Self {
            handle,
            _drop_tx: drop_tx,
            _thread: Some(join_handle),
            channels: Mutex::new(HashMap::new()),
            master_volume: Mutex::new(1.0),
        })
    }

    /// Open or resume a channel by its identifier, source file path, and volume.
    pub fn play_channel(
        &self,
        channel: &ChannelId,
        name: &str,
        source_path: &str,
        volume: f32,
    ) -> Result<(), AudioError> {
        self.play_channel_ext(channel, name, source_path, volume, true)
    }

    /// Full play_channel with customizable loop setting.
    pub fn play_channel_ext(
        &self,
        channel: &ChannelId,
        name: &str,
        source_path: &str,
        volume: f32,
        loop_enabled: bool,
    ) -> Result<(), AudioError> {
        let master = *self.master_volume.lock();
        let mut channels = self.channels.lock();

        // If channel is already playing, adjust volume and ensure resumed
        if let Some(state) = channels.get_mut(channel) {
            state.gain = volume.clamp(0.0, 1.0);
            if let Some(sink) = &state.sink {
                sink.set_volume(effective_volume(state.gain, state.muted, master) * SPATIAL_FACTOR);
                sink.play();
            }
            return Ok(());
        }

        // Limit channels to MAX_CHANNELS
        if channels.len() >= MAX_CHANNELS
            && let Some(oldest) = channels.keys().next().cloned()
            && let Some(state) = channels.remove(&oldest)
            && let Some(sink) = state.sink
        {
            sink.stop();
        }

        let sink = SpatialSink::try_new(
            &self.handle,
            [0.0, 0.0, 1.0],
            [-1.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
        )
        .map_err(|e| AudioError::NoDevice(e.to_string()))?;

        let source_exists = !source_path.is_empty() && std::path::Path::new(source_path).exists();
        if source_exists {
            let file = File::open(source_path).map_err(|e| AudioError::Open(e.to_string()))?;
            let decoder = Decoder::new(BufReader::new(file))
                .map_err(|e| AudioError::Decode(e.to_string()))?;
            if loop_enabled {
                sink.append(decoder.repeat_infinite());
            } else {
                sink.append(decoder);
            }
        } else {
            // Procedural ambient sound generator for built-in channels
            let synth = AmbientSoundSource::new(channel.as_str(), 44100);
            sink.append(synth);
        }

        let gain = volume.clamp(0.0, 1.0);
        sink.set_volume(effective_volume(gain, false, master) * SPATIAL_FACTOR);

        channels.insert(
            channel.clone(),
            ChannelState {
                sink: Some(sink),
                config: ChannelConfig {
                    channel: channel.clone(),
                    name: name.to_string(),
                    source_path: source_path.to_string(),
                    loop_enabled,
                },
                gain,
                muted: false,
                pan: 0.0,
            },
        );

        Ok(())
    }

    /// Legacy open_channel: opens with generated UUID channel id.
    pub fn open_channel(
        &self,
        name: &str,
        source_path: &str,
        loop_enabled: bool,
    ) -> Result<ChannelId, AudioError> {
        let channel = ChannelId::new();
        self.play_channel_ext(&channel, name, source_path, DEFAULT_GAIN, loop_enabled)?;
        Ok(channel)
    }

    /// Stop and remove a channel.
    pub fn stop_channel(&self, channel: &ChannelId) -> Result<(), AudioError> {
        let mut channels = self.channels.lock();
        if let Some(state) = channels.remove(channel)
            && let Some(sink) = state.sink
        {
            sink.stop();
        }
        Ok(())
    }

    /// Set gain for a specific channel.
    pub fn set_channel_gain(&self, channel: &ChannelId, gain: f32) {
        let master = *self.master_volume.lock();
        let mut channels = self.channels.lock();
        if let Some(state) = channels.get_mut(channel)
            && let Some(sink) = &mut state.sink
        {
            state.gain = gain.clamp(0.0, 1.0);
            sink.set_volume(effective_volume(state.gain, state.muted, master) * SPATIAL_FACTOR);
        }
    }

    /// Set volume alias for set_channel_gain.
    pub fn set_channel_volume(&self, channel: &ChannelId, volume: f32) {
        self.set_channel_gain(channel, volume);
    }

    /// Spatial audio pan (-1.0 = left, 0.0 = center, +1.0 = right).
    pub fn set_channel_pan(&self, channel: &ChannelId, pan: f32) {
        let clamped = pan.clamp(-1.0, 1.0);
        let mut channels = self.channels.lock();
        if let Some(state) = channels.get_mut(channel) {
            state.pan = clamped;
            if let Some(sink) = &state.sink {
                sink.set_emitter_position([clamped, 0.0, 1.0]);
            }
        }
    }

    pub fn set_channel_muted(&self, channel: &ChannelId, muted: bool) {
        let master = *self.master_volume.lock();
        let mut channels = self.channels.lock();
        if let Some(state) = channels.get_mut(channel)
            && let Some(sink) = &mut state.sink
        {
            state.muted = muted;
            sink.set_volume(effective_volume(state.gain, state.muted, master) * SPATIAL_FACTOR);
        }
    }

    pub fn set_master_volume(&self, volume: f32) {
        let vol = volume.clamp(0.0, 1.0);
        *self.master_volume.lock() = vol;
        let channels = self.channels.lock();
        for state in channels.values() {
            if let Some(sink) = &state.sink {
                sink.set_volume(effective_volume(state.gain, state.muted, vol) * SPATIAL_FACTOR);
            }
        }
    }

    pub fn pause_channel(&self, channel: &ChannelId) {
        let channels = self.channels.lock();
        if let Some(state) = channels.get(channel)
            && let Some(sink) = &state.sink
        {
            sink.pause();
        }
    }

    pub fn resume_channel(&self, channel: &ChannelId) {
        let channels = self.channels.lock();
        if let Some(state) = channels.get(channel)
            && let Some(sink) = &state.sink
        {
            sink.play();
        }
    }

    pub fn close_channel(&self, channel: &ChannelId) {
        let _ = self.stop_channel(channel);
    }

    pub fn close_all(&self) {
        let mut channels = self.channels.lock();
        for (_, state) in channels.drain() {
            if let Some(sink) = state.sink {
                sink.stop();
            }
        }
    }

    pub fn channel_count(&self) -> usize {
        self.channels.lock().len()
    }

    pub fn list_active_channel_ids(&self) -> Vec<String> {
        self.channels
            .lock()
            .keys()
            .map(|k| k.0.clone())
            .collect()
    }

    pub fn is_channel_active(&self, channel: &ChannelId) -> bool {
        self.channels.lock().contains_key(channel)
    }

    pub fn active_configs(&self) -> Vec<ChannelConfig> {
        self.channels
            .lock()
            .values()
            .filter_map(|s| s.sink.as_ref().map(|_| s.config.clone()))
            .collect()
    }
}

/// Procedural ambient sound generator for continuous playback without media files.
pub struct AmbientSoundSource {
    sound_type: String,
    sample_rate: u32,
    sample_index: u64,
    rng_state: u32,
    last_brown: f32,
}

impl AmbientSoundSource {
    pub fn new(sound_type: &str, sample_rate: u32) -> Self {
        // Derive initial seed from sound_type hash
        let mut seed: u32 = 0x811c9dc5;
        for b in sound_type.bytes() {
            seed ^= b as u32;
            seed = seed.wrapping_mul(0x01000193);
        }
        if seed == 0 {
            seed = 0x12345678;
        }

        Self {
            sound_type: sound_type.to_string(),
            sample_rate,
            sample_index: 0,
            rng_state: seed,
            last_brown: 0.0,
        }
    }
}

impl Iterator for AmbientSoundSource {
    type Item = f32;

    fn next(&mut self) -> Option<f32> {
        self.sample_index = self.sample_index.wrapping_add(1);
        let t = self.sample_index as f32 / self.sample_rate as f32;

        // XorShift PRNG for white noise
        self.rng_state ^= self.rng_state << 13;
        self.rng_state ^= self.rng_state >> 17;
        self.rng_state ^= self.rng_state << 5;
        let white = (self.rng_state as f32 / u32::MAX as f32) * 2.0 - 1.0;

        // Brown noise filter (integrated random walk)
        self.last_brown = (self.last_brown + 0.03 * white) / 1.02;
        let brown = self.last_brown.clamp(-1.0, 1.0);

        let out = match self.sound_type.as_str() {
            "ch-rain" => {
                let patter = if white > 0.985 { white * 0.8 } else { 0.0 };
                brown * 0.4 + white * 0.15 + patter
            }
            "ch-waves" => {
                let swell = ((t * 2.0 * std::f32::consts::PI / 8.0).sin() * 0.5 + 0.5).powf(2.0);
                brown * (0.2 + 0.8 * swell)
            }
            "ch-birds" => {
                let cycle = t % 3.5;
                let chirp = if cycle < 0.25 {
                    let freq = 2400.0 + 800.0 * (cycle * 20.0).sin();
                    (t * 2.0 * std::f32::consts::PI * freq).sin() * 0.35
                } else if (0.4..0.6).contains(&cycle) {
                    let freq = 2800.0 + 600.0 * (cycle * 25.0).sin();
                    (t * 2.0 * std::f32::consts::PI * freq).sin() * 0.25
                } else {
                    0.0
                };
                brown * 0.08 + chirp
            }
            "ch-cafe" => {
                let murmur = (t * 2.0 * std::f32::consts::PI * 180.0).sin() * 0.08
                    + (t * 2.0 * std::f32::consts::PI * 260.0).sin() * 0.05;
                let clink = if white > 0.9992 { (t * 2.0 * std::f32::consts::PI * 3400.0).sin() * 0.4 } else { 0.0 };
                brown * 0.35 + murmur + clink
            }
            "ch-fire" => {
                let crackle = if white > 0.993 { white * 0.7 } else { 0.0 };
                brown * 0.45 + crackle
            }
            "ch-wind" => {
                let gust = (t * 2.0 * std::f32::consts::PI / 5.2).sin() * 0.4 + 0.6;
                brown * 0.35 * gust + white * 0.1 * gust
            }
            "ch-stream" => {
                let bubble = ((t * 14.0).sin() * 0.3 + 0.7) * (white * 0.25);
                brown * 0.25 + bubble
            }
            "ch-noise" => {
                brown * 0.6
            }
            "ch-binaural" => {
                (t * 2.0 * std::f32::consts::PI * 210.0).sin() * 0.3
            }
            "ch-spatial" => {
                let lfo = (t * 2.0 * std::f32::consts::PI * 0.2).sin();
                let tone = (t * 2.0 * std::f32::consts::PI * (320.0 + 40.0 * lfo)).sin() * 0.25;
                brown * 0.2 + tone
            }
            _ => brown * 0.5,
        };

        Some(out.clamp(-1.0, 1.0))
    }
}

impl Source for AmbientSoundSource {
    fn current_frame_len(&self) -> Option<usize> {
        None
    }

    fn channels(&self) -> u16 {
        1
    }

    fn sample_rate(&self) -> u32 {
        self.sample_rate
    }

    fn total_duration(&self) -> Option<std::time::Duration> {
        None
    }
}

/// Pure gain math, extracted so it is testable without an audio device.
pub fn effective_volume(gain: f32, muted: bool, master: f32) -> f32 {
    if muted {
        0.0
    } else {
        gain.clamp(0.0, 1.0) * master.clamp(0.0, 1.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn muting_forces_silence() {
        assert_eq!(effective_volume(0.8, true, 1.0), 0.0);
    }

    #[test]
    fn gain_scales_with_master() {
        assert!((effective_volume(0.5, false, 1.0) - 0.5).abs() < f32::EPSILON);
        assert!((effective_volume(0.5, false, 0.5) - 0.25).abs() < f32::EPSILON);
    }

    #[test]
    fn inputs_are_clamped_to_unit_range() {
        assert!((effective_volume(5.0, false, 5.0) - 1.0).abs() < f32::EPSILON);
        assert_eq!(effective_volume(-1.0, false, 1.0), 0.0);
    }

    #[test]
    fn channel_id_from_and_display() {
        let id1 = ChannelId::from("ch-rain");
        assert_eq!(id1.as_str(), "ch-rain");
        assert_eq!(format!("{id1}"), "ch-rain");

        let id2 = ChannelId::from_string("ch-waves".to_string());
        assert_eq!(id2.as_str(), "ch-waves");

        let id3 = ChannelId::new();
        assert!(!id3.as_str().is_empty());
    }

    #[test]
    fn audio_engine_playback_workflow() {
        if let Ok(engine) = AudioEngine::new() {
            let tmp = std::env::temp_dir().join(format!("cacalm-audio-test-{}", uuid::Uuid::now_v7()));
            let default_sounds = crate::audio::synth::ensure_default_sounds(&tmp)
                .expect("synth default sounds should succeed");
            assert!(!default_sounds.is_empty());

            let (id, path) = &default_sounds[0];
            let ch_id = ChannelId::from(id.as_str());

            // 1. Play channel
            if let Err(e) = engine.play_channel(&ch_id, "Rain", path.to_str().unwrap(), 0.6) {
                panic!("play_channel error: {e:?}");
            }
            assert_eq!(engine.channel_count(), 1);
            assert!(engine.is_channel_active(&ch_id));

            // 2. Change volume
            engine.set_channel_volume(&ch_id, 0.9);

            // 3. Change spatial pan
            engine.set_channel_pan(&ch_id, -0.5);

            // 4. Change master volume
            engine.set_master_volume(0.7);

            // 5. Active channel list
            let active = engine.list_active_channel_ids();
            assert_eq!(active, vec![id.clone()]);

            // 6. Stop channel
            assert!(engine.stop_channel(&ch_id).is_ok());
            assert_eq!(engine.channel_count(), 0);
            assert!(!engine.is_channel_active(&ch_id));

            let _ = std::fs::remove_dir_all(&tmp);
        }
    }
}
