//! Multi-channel ambient audio engine.
//!
//! Design notes (see PRD §F1):
//! - Up to [`MAX_CHANNELS`] simultaneous channels, each backed by a `rodio::Sink`.
//! - Per-channel gain and a single master gain.
//! - The heavy DSP work (30 ms loop crossfade, procedural generators) lands in
//!   later phases; this module owns the playback/threading contract.

use parking_lot::Mutex;
use rodio::{Decoder, OutputStream, OutputStreamHandle, Sink};
use std::collections::HashMap;
use std::fs::File;
use std::io::BufReader;
use uuid::Uuid;

/// Opaque channel handle.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ChannelId(pub Uuid);

impl ChannelId {
    pub fn new() -> Self {
        Self(Uuid::now_v7())
    }
}

impl Default for ChannelId {
    fn default() -> Self {
        Self::new()
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
    sink: Sink,
    gain: f32,
    muted: bool,
    config: ChannelConfig,
}

pub const MAX_CHANNELS: usize = 8;
pub const DEFAULT_GAIN: f32 = 0.8;

/// Multi-channel playback engine. Cheap to clone via `Arc` at the app layer.
pub struct AudioEngine {
    handle: OutputStreamHandle,
    /// Keeps the output stream alive for the engine's lifetime.
    _stream: OutputStream,
    channels: Mutex<HashMap<ChannelId, ChannelState>>,
    master_volume: Mutex<f32>,
}

/// Errors surfaced to the UI. Kept stringly for Tauri IPC simplicity.
#[derive(Debug, thiserror::Error)]
pub enum AudioError {
    #[error("no audio output device is available: {0}")]
    NoDevice(String),
    #[error("cannot open file: {0}")]
    Open(String),
    #[error("unsupported or corrupt audio: {0}")]
    Decode(String),
}

impl AudioEngine {
    pub fn new() -> Result<Self, AudioError> {
        let (stream, handle) =
            OutputStream::try_default().map_err(|e| AudioError::NoDevice(e.to_string()))?;
        Ok(Self {
            handle,
            _stream: stream,
            channels: Mutex::new(HashMap::new()),
            master_volume: Mutex::new(1.0),
        })
    }

    /// Open (or replace) a channel from a local file path and begin playback.
    pub fn open_channel(
        &self,
        name: &str,
        source_path: &str,
        loop_enabled: bool,
    ) -> Result<ChannelId, AudioError> {
        let mut channels = self.channels.lock();

        // FIFO eviction keeps us under the cap without surfacing an error to the UI.
        if channels.len() >= MAX_CHANNELS {
            if let Some(oldest) = channels.keys().next().cloned() {
                if let Some(state) = channels.remove(&oldest) {
                    state.sink.stop();
                }
            }
        }

        let file = File::open(source_path).map_err(|e| AudioError::Open(e.to_string()))?;
        let decoder = Decoder::new(BufReader::new(file))
            .map_err(|e| AudioError::Decode(e.to_string()))?;

        let sink = Sink::try_new(&self.handle).map_err(|e| AudioError::NoDevice(e.to_string()))?;
        sink.set_loop(loop_enabled);
        sink.append(decoder);

        let channel = ChannelId::new();
        let gain = DEFAULT_GAIN;
        let master = *self.master_volume.lock();
        sink.set_volume(gain * master);

        channels.insert(
            channel.clone(),
            ChannelState {
                sink,
                gain,
                muted: false,
                config: ChannelConfig {
                    channel: channel.clone(),
                    name: name.to_string(),
                    source_path: source_path.to_string(),
                    loop_enabled,
                },
            },
        );
        Ok(channel)
    }

    pub fn set_channel_gain(&self, channel: &ChannelId, gain: f32) {
        let master = *self.master_volume.lock();
        let mut channels = self.channels.lock();
        if let Some(state) = channels.get_mut(channel) {
            state.gain = gain.clamp(0.0, 1.0);
            state.sink.set_volume(state.effective_volume(master));
        }
    }

    pub fn set_channel_muted(&self, channel: &ChannelId, muted: bool) {
        let master = *self.master_volume.lock();
        let mut channels = self.channels.lock();
        if let Some(state) = channels.get_mut(channel) {
            state.muted = muted;
            state.sink.set_volume(state.effective_volume(master));
        }
    }

    pub fn set_master_volume(&self, volume: f32) {
        let vol = volume.clamp(0.0, 1.0);
        *self.master_volume.lock() = vol;
        let channels = self.channels.lock();
        for state in channels.values() {
            state.sink.set_volume(state.effective_volume(vol));
        }
    }

    pub fn pause_channel(&self, channel: &ChannelId) {
        if let Some(state) = self.channels.lock().get(channel) {
            state.sink.pause();
        }
    }

    pub fn resume_channel(&self, channel: &ChannelId) {
        if let Some(state) = self.channels.lock().get(channel) {
            state.sink.play();
        }
    }

    /// Stop playback and drop the channel.
    pub fn close_channel(&self, channel: &ChannelId) {
        if let Some(state) = self.channels.lock().remove(channel) {
            state.sink.stop();
        }
    }

    pub fn close_all(&self) {
        let mut channels = self.channels.lock();
        for (_, state) in channels.drain() {
            state.sink.stop();
        }
    }

    pub fn channel_count(&self) -> usize {
        self.channels.lock().len()
    }

    pub fn active_configs(&self) -> Vec<ChannelConfig> {
        self.channels
            .lock()
            .values()
            .map(|s| s.config.clone())
            .collect()
    }
}

impl ChannelState {
    fn effective_volume(&self, master: f32) -> f32 {
        if self.muted {
            0.0
        } else {
            self.gain * master
        }
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
    use super::effective_volume;

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
}
