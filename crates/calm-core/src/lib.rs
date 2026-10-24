pub mod audio;
pub mod library;
pub mod preset;
pub mod timer;
pub mod paths;
pub mod pro;

pub use audio::{AudioEngine, ChannelId};
pub use timer::{PomodoroTimer, Phase, TimerEvent, TimerConfig};
pub use library::{SoundLibrary, SoundEntry};
pub use preset::{PresetManager, Preset};
pub use pro::{
    activate_license, deactivate_license, get_pro_status, verify_license, ProStatus,
};