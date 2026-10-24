use calm_core::{AudioEngine, PomodoroTimer, PresetManager, SoundLibrary};
use std::sync::Mutex;
use tauri::Manager;

pub mod commands;
pub mod settings;
pub mod shortcuts;
pub mod tray;

pub use commands::{
    activate_pro_license, deactivate_pro_license, delete_preset, delete_sound, get_app_version,
    get_pro_status, get_settings, get_timer_snapshot, import_sound, list_active_channels,
    list_presets, list_sounds, load_preset, minimize_to_tray, open_main_window, play_channel,
    save_preset, set_channel_pan, set_channel_volume, set_master_volume, set_shortcuts_enabled,
    set_tray_enabled, stop_channel, timer_pause, timer_reset, timer_resume, timer_skip,
    timer_tick, toggle_shortcuts, toggle_timer_state, toggle_tray, update_settings,
};

/// Tauri managed state — holds the live engine instances and app settings.
pub struct AppState {
    pub engine: Result<AudioEngine, String>,
    pub timer: Mutex<PomodoroTimer>,
    pub library: Result<SoundLibrary, String>,
    pub preset_mgr: Result<PresetManager, String>,
    pub settings: Mutex<settings::AppSettings>,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            engine: AudioEngine::new().map_err(|e| e.to_string()),
            timer: Mutex::new(PomodoroTimer::default()),
            library: SoundLibrary::open().map_err(|e| e.to_string()),
            preset_mgr: PresetManager::new().map_err(|e| e.to_string()),
            settings: Mutex::new(settings::load_settings()),
        }
    }
}

impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let builder = tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_process::init())
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .manage(AppState::new())
        .setup(|app| {
            let handle = app.handle();
            let settings = {
                let state = handle.state::<AppState>();
                state
                    .settings
                    .lock()
                    .map(|s| s.clone())
                    .unwrap_or_default()
            };

            if let Err(e) = tray::setup_tray(handle, &settings) {
                eprintln!("Failed to setup system tray: {e}");
            }

            if let Err(e) = shortcuts::register_global_shortcuts(handle, &settings) {
                eprintln!("Failed to register global shortcuts: {e}");
            }

            {
                let state = handle.state::<AppState>();
                if let Ok(ref lib) = state.library {
                    let _ = lib.seed_default_sounds();
                }
            }

            Ok(())
        })
        .on_window_event(|window, event| {
            if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                let app = window.app_handle();
                let state = app.state::<AppState>();
                let should_minimize = state
                    .settings
                    .lock()
                    .map(|s| s.minimize_to_tray && s.tray_enabled)
                    .unwrap_or(false);

                if should_minimize {
                    api.prevent_close();
                    let _ = window.hide();
                }
            }
        })
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
            commands::get_app_version,
            commands::get_settings,
            commands::update_settings,
            commands::toggle_tray,
            commands::toggle_shortcuts,
            commands::set_shortcuts_enabled,
            commands::set_tray_enabled,
            commands::minimize_to_tray,
            commands::play_channel,
            commands::stop_channel,
            commands::set_channel_volume,
            commands::set_master_volume,
            commands::set_channel_pan,
            commands::list_active_channels,
            commands::get_pro_status,
            commands::activate_pro_license,
            commands::deactivate_pro_license
        ]);

    builder
        .run(tauri::generate_context!())
        .expect("error while running CACalm");
}
