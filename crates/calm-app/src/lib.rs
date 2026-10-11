use calm_core::{AudioEngine, PomodoroTimer, PresetManager, SoundLibrary};
use serde::Serialize;
use std::sync::Mutex;

#[derive(Serialize, Clone)]
pub struct TimerSnapshot {
    pub phase: String,
    pub remaining: u64,
    pub paused: bool,
    pub sessions_today: u32,
}

/// Tauri managed state — holds the live engine instances.
pub struct AppState {
    pub engine: Result<AudioEngine, String>,
    pub timer: Mutex<PomodoroTimer>,
    pub library: Result<SoundLibrary, String>,
    pub preset_mgr: Result<PresetManager, String>,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            engine: AudioEngine::new().map_err(|e| e.to_string()),
            timer: Mutex::new(PomodoroTimer::default()),
            library: SoundLibrary::open().map_err(|e| e.to_string()),
            preset_mgr: PresetManager::new().map_err(|e| e.to_string()),
        }
    }
}

impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
}

/// Get current timer state (placeholder — will be wired to real timer in T3)
#[tauri::command]
fn get_timer_snapshot(state: tauri::State<'_, AppState>) -> Result<TimerSnapshot, String> {
    let timer = state
        .timer
        .lock()
        .map_err(|e| e.to_string())?;
    Ok(TimerSnapshot {
        phase: timer.phase().label().to_string(),
        remaining: timer.remaining().as_secs(),
        paused: timer.is_paused(),
        sessions_today: timer.sessions_today,
    })
}