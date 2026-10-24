use parking_lot::Mutex;
use rodio::{Decoder, OutputStream, OutputStreamHandle, Sink, Source};
use std::collections::HashMap;
use std::fs::File;
use std::io::BufReader;
use std::sync::mpsc;
use std::thread;
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
    sink: Option<Sink>,
    config: ChannelConfig,
    gain: f32,
    muted: bool,
}

pub const MAX_CHANNELS: usize = 8;
pub const DEFAULT_GAIN: f32 = 0.8;

#[derive(Debug, thiserror::Error)]
pub enum AudioError {
    #[error("no audio output device is available: {0}")]
    NoDevice(String),
    #[error("cannot open file: {0}")]
    Open(String),
    #[error("unsupported or corrupt audio: {0}")]
    Decode(String),
}

/// Multi-channel playback engine.
///
/// `rodio::OutputStream` is `!Send + !Sync` on Linux/ALSA (the stream object
/// must stay on the thread that created it), so it is created and owned by a
/// dedicated "stream owner" worker thread. Only the `OutputStreamHandle` (which
/// *is* `Send + Sync`) crosses the thread boundary to `AudioEngine`. When
/// `AudioEngine` is dropped, `_drop_tx` hangs up, causing the worker thread
/// to unpark and drop `OutputStream`.
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

    pub fn open_channel(
        &self,
        name: &str,
        source_path: &str,
        loop_enabled: bool,
    ) -> Result<ChannelId, AudioError> {
        let mut channels = self.channels.lock();

        if channels.len() >= MAX_CHANNELS
            && let Some(oldest) = channels.keys().next().cloned()
            && let Some(state) = channels.remove(&oldest)
            && let Some(sink) = state.sink
        {
            sink.stop();
        }

        let file = File::open(source_path).map_err(|e| AudioError::Open(e.to_string()))?;
        let decoder = Decoder::new(BufReader::new(file))
            .map_err(|e| AudioError::Decode(e.to_string()))?;

        let sink = Sink::try_new(&self.handle)
            .map_err(|e| AudioError::NoDevice(e.to_string()))?;
        if loop_enabled {
            sink.append(decoder.repeat_infinite());
        } else {
            sink.append(decoder);
        }

        let channel = ChannelId::new();
        let gain = DEFAULT_GAIN;
        let master = *self.master_volume.lock();
        sink.set_volume(gain * master);

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
            },
        );
        Ok(channel)
    }

    pub fn set_channel_gain(&self, channel: &ChannelId, gain: f32) {
        let master = *self.master_volume.lock();
        let mut channels = self.channels.lock();
        if let Some(state) = channels.get_mut(channel)
            && let Some(sink) = &mut state.sink
        {
            state.gain = gain.clamp(0.0, 1.0);
            sink.set_volume(effective_volume(state.gain, state.muted, master));
        }
    }

    pub fn set_channel_muted(&self, channel: &ChannelId, muted: bool) {
        let master = *self.master_volume.lock();
        let mut channels = self.channels.lock();
        if let Some(state) = channels.get_mut(channel)
            && let Some(sink) = &mut state.sink
        {
            state.muted = muted;
            sink.set_volume(effective_volume(state.gain, state.muted, master));
        }
    }

    pub fn set_master_volume(&self, volume: f32) {
        let vol = volume.clamp(0.0, 1.0);
        *self.master_volume.lock() = vol;
        let channels = self.channels.lock();
        for state in channels.values() {
            if let Some(sink) = &state.sink {
                sink.set_volume(effective_volume(state.gain, state.muted, vol));
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
        let mut channels = self.channels.lock();
        if let Some(state) = channels.remove(channel)
            && let Some(sink) = state.sink
        {
            sink.stop();
        }
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

    pub fn active_configs(&self) -> Vec<ChannelConfig> {
        self.channels
            .lock()
            .values()
            .filter_map(|s| s.sink.as_ref().map(|_| s.config.clone()))
            .collect()
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