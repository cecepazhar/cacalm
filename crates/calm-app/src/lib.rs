use calm_core::{AudioEngine, PomodoroTimer, PresetManager, SoundLibrary};
use std::sync::Mutex;

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

mod commands;

pub use commands::get_timer_snapshot;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let mut builder = tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_process::init())
        .manage(AppState::new())
        .invoke_handler(tauri::generate_handler![commands::get_timer_snapshot]);

    builder
        .run(tauri::generate_context!())
        .expect("error while running CACalm");
}

