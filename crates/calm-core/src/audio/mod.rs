//! Multi-channel ambient audio engine.
//!
//! Design notes (see PRD §F1):
//! - Up to [`MAX_CHANNELS`] simultaneous channels, each backed by a `rodio::SpatialSink`.
//! - Per-channel gain, spatial pan, and a single master gain.
//! - Seamless loop-point smoothing and built-in procedural ambient generators.
//!
//! Feature gating:
//! - `audio` feature → real playback via `rodio`.
//! - otherwise → API-compatible stub; compile-time proof of the surface.

pub mod synth;

#[cfg(feature = "audio")]
mod audio_impl;
#[cfg(not(feature = "audio"))]
mod audio_stub;

#[cfg(feature = "audio")]
pub use audio_impl::{AudioEngine, AudioError, ChannelConfig, ChannelId, MAX_CHANNELS, SPATIAL_FACTOR};
#[cfg(not(feature = "audio"))]
pub use audio_stub::{AudioEngine, AudioError, ChannelConfig, ChannelId, MAX_CHANNELS, SPATIAL_FACTOR};
