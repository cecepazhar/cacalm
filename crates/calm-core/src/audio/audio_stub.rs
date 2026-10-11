use parking_lot::Mutex;
use std::collections::HashMap;
use uuid::Uuid;

/// Stub implementation when `audio` feature is disabled.
/// Returns errors on any audio call so the compiler still verifies the API surface.

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

#[derive(Debug, Clone)]
pub struct ChannelConfig {
    pub channel: ChannelId,
    pub name: String,
    pub source_path: String,
    pub loop_enabled: bool,
}

pub const MAX_CHANNELS: usize = 8;

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

    pub fn open_channel(&self, _name: &str, _source_path: &str, _loop_enabled: bool) -> Result<ChannelId, AudioError> {
        Err(AudioError::NoAudio)
    }

    pub fn set_channel_gain(&self, _channel: &ChannelId, _gain: f32) {}
    pub fn set_channel_muted(&self, _channel: &ChannelId, _muted: bool) {}
    pub fn set_master_volume(&self, _volume: f32) {}
    pub fn pause_channel(&self, _channel: &ChannelId) {}
    pub fn resume_channel(&self, _channel: &ChannelId) {}
    pub fn close_channel(&self, _channel: &ChannelId) {}
    pub fn close_all(&self) {}
    pub fn channel_count(&self) -> usize {
        0
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