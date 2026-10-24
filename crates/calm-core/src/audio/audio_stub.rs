use uuid::Uuid;

/// Stub implementation when `audio` feature is disabled.
/// Returns errors on any audio call so the compiler still verifies the API surface.

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

#[derive(Debug, Clone)]
pub struct ChannelConfig {
    pub channel: ChannelId,
    pub name: String,
    pub source_path: String,
    pub loop_enabled: bool,
}

pub const MAX_CHANNELS: usize = 8;
pub const DEFAULT_GAIN: f32 = 0.8;
pub const SPATIAL_FACTOR: f32 = 2.0;

#[derive(Debug, thiserror::Error)]
pub enum AudioError {
    #[error("audio backend disabled in this build")]
    NoAudio,
    #[error("cannot open file: {0}")]
    Open(String),
    #[error("unsupported or corrupt audio: {0}")]
    Decode(String),
}

pub struct AudioEngine;

impl AudioEngine {
    pub fn new() -> Result<Self, AudioError> {
        Err(AudioError::NoAudio)
    }

    pub fn play_channel(
        &self,
        _channel: &ChannelId,
        _name: &str,
        _source_path: &str,
        _volume: f32,
    ) -> Result<(), AudioError> {
        Err(AudioError::NoAudio)
    }

    pub fn play_channel_ext(
        &self,
        _channel: &ChannelId,
        _name: &str,
        _source_path: &str,
        _volume: f32,
        _loop_enabled: bool,
    ) -> Result<(), AudioError> {
        Err(AudioError::NoAudio)
    }

    pub fn open_channel(
        &self,
        _name: &str,
        _source_path: &str,
        _loop_enabled: bool,
    ) -> Result<ChannelId, AudioError> {
        Err(AudioError::NoAudio)
    }

    pub fn stop_channel(&self, _channel: &ChannelId) -> Result<(), AudioError> {
        Ok(())
    }

    pub fn set_channel_gain(&self, _channel: &ChannelId, _gain: f32) {}
    pub fn set_channel_volume(&self, _channel: &ChannelId, _volume: f32) {}
    pub fn set_channel_pan(&self, _channel: &ChannelId, _pan: f32) {}
    pub fn set_channel_muted(&self, _channel: &ChannelId, _muted: bool) {}
    pub fn set_master_volume(&self, _volume: f32) {}
    pub fn pause_channel(&self, _channel: &ChannelId) {}
    pub fn resume_channel(&self, _channel: &ChannelId) {}
    pub fn close_channel(&self, _channel: &ChannelId) {}
    pub fn close_all(&self) {}
    pub fn channel_count(&self) -> usize {
        0
    }
    pub fn list_active_channel_ids(&self) -> Vec<String> {
        Vec::new()
    }
    pub fn is_channel_active(&self, _channel: &ChannelId) -> bool {
        false
    }
    pub fn active_configs(&self) -> Vec<ChannelConfig> {
        Vec::new()
    }
}

pub fn effective_volume(gain: f32, muted: bool, master: f32) -> f32 {
    if muted {
        0.0
    } else {
        gain.clamp(0.0, 1.0) * master.clamp(0.0, 1.0)
    }
}
