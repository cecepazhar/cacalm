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

pub use commands::{
    delete_preset, delete_sound, get_app_version, get_timer_snapshot, import_sound, list_presets,
    list_sounds, load_preset, save_preset, timer_pause, timer_reset, timer_resume, timer_skip,
    timer_tick,
};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let mut builder = tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_process::init())
        .manage(AppState::new())
        .invoke_handler(tauri::generate_handler![
            commands::get_timer_snapshot,
            commands::timer_pause,
            commands::timer_resume,
            commands::timer_reset,
            commands::timer_skip,
            commands::timer_tick,
            commands::list_sounds,
            commands::import_sound,
            commands::delete_sound,
            commands::list_presets,
            commands::save_preset,
            commands::load_preset,
            commands::delete_preset,
            commands::get_app_version
        ]);

    builder
        .run(tauri::generate_context!())
        .expect("error while running CACalm");
}

