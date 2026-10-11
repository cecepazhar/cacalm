use calm_core::{AudioEngine, ChannelId, PomodoroTimer, TimerConfig, SoundLibrary, PresetManager};
use tauri::State;
use serde::Serialize;

#[derive(Serialize, Clone)]
pub struct TimerSnapshot {
    pub phase: String,
    pub remaining: u64,
    pub paused: bool,
    pub sessions_today: u32,
}

#[derive(Default)]
pub struct App {
    engine: Option<AudioEngine>,
    timer: PomodoroTimer,
    library: Option<SoundLibrary>,
    preset_mgr: Option<PresetManager>,
}

impl App {
    pub fn new() -> Self {
        Self {
            engine: AudioEngine::new().ok(),
            timer: PomodoroTimer::default(),
            library: SoundLibrary::open().ok(),
            preset_mgr: PresetManager::new().ok(),
        }
    }
}

// Tauri commands (T2-T7) will be added here as they are implemented.
#[tauri::command]
fn get_timer_snapshot() -> TimerSnapshot {
    // Placeholder — real implementation in T3
    TimerSnapshot {
        phase: "focus".into(),
        remaining: 1500,
        paused: false,
        sessions_today: 0,
    }
}
