//! Multi-channel ambient audio engine.
//!
//! Design notes (see PRD §F1):
//! - Up to [`MAX_CHANNELS`] simultaneous channels, each backed by a `rodio::Sink`.
//! - Per-channel gain and a single master gain.
//! - Crossfade/loop-point smoothing and procedural generators land in later
//!   phases; this module owns the playback/threading contract now.
//!
//! Feature gating:
//! - `audio` feature → real playback via `rodio`.
//! - otherwise → API-compatible stub; compile-time proof of the surface.

#[cfg(feature = "audio")]
mod audio_impl;
#[cfg(not(feature = "audio"))]
mod audio_stub;

#[cfg(feature = "audio")]
pub use audio_impl::{AudioEngine, AudioError, ChannelConfig, ChannelId, MAX_CHANNELS};
#[cfg(not(feature = "audio"))]
pub use audio_stub::{AudioEngine, AudioError, ChannelConfig, ChannelId, MAX_CHANNELS};