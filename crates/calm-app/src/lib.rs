use calm_core::{PomodoroTimer, PresetManager, SoundLibrary};
use std::sync::Mutex;
use std::sync::atomic::{AtomicBool, Ordering};
use tauri::Manager;

pub mod commands;
pub mod memory;
pub mod settings;
pub mod shortcuts;
pub mod tray;

pub use commands::{
    delete_preset, delete_sound, get_app_version, get_settings, get_timer_snapshot, import_sound,
    list_presets, list_sounds, load_preset, minimize_to_tray, open_external, set_sound_playing, open_main_window, read_sound_file,
    save_preset, set_shortcuts_enabled, set_tray_enabled, timer_pause, timer_reset, timer_resume,
    timer_skip, timer_tick, toggle_shortcuts, toggle_timer_state, toggle_tray, update_settings,
    window_close, window_is_maximized, window_maximize, window_minimize, window_start_dragging,
};

/// Set when the user explicitly quits, so the close-to-tray handler lets the
/// window close instead of hiding it.
pub static QUITTING: AtomicBool = AtomicBool::new(false);

/// Whether the WebView currently has sounds playing (reported by the UI).
pub static SOUND_PLAYING: AtomicBool = AtomicBool::new(false);

/// Fully quits CACalm (tray "Quit").
pub fn quit_app<R: tauri::Runtime>(app: &tauri::AppHandle<R>) {
    QUITTING.store(true, Ordering::SeqCst);
    for (_, w) in app.webview_windows() {
        let _ = w.destroy();
    }
    app.exit(0);
}

/// Tauri managed state — holds the live engine instances and app settings.
pub struct AppState {
    pub timer: Mutex<PomodoroTimer>,
    pub library: Result<SoundLibrary, String>,
    pub preset_mgr: Result<PresetManager, String>,
    pub settings: Mutex<settings::AppSettings>,
}

impl AppState {
    pub fn new() -> Self {
        Self {
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
        // A second launch focuses the running CACalm instead of starting another audio engine.
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            open_main_window(app);
        }))
        // Remembers size, position and maximized state. First launch uses tauri.conf.json
        // (maximized). VISIBLE is excluded: the window stays hidden until the boot splash
        // has painted, so the user never sees a blank WebView.
        .plugin(
            tauri_plugin_window_state::Builder::default()
                .with_state_flags(
                    tauri_plugin_window_state::StateFlags::all()
                        & !tauri_plugin_window_state::StateFlags::VISIBLE,
                )
                .build(),
        )
        // Show the (hidden) main window once its first page — the splash — has loaded.
        .on_page_load(|webview, payload| {
            if matches!(payload.event(), tauri::webview::PageLoadEvent::Finished)
                && webview.label() == "main"
            {
                let window = webview.window();
                let _ = window.show();
                let _ = window.set_focus();
            }
        })
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

            // Fallback: show the window even if the page-load event never arrives.
            if let Some(window) = handle.get_webview_window("main") {
                memory::install(&window);
                tauri::async_runtime::spawn(async move {
                    tokio::time::sleep(std::time::Duration::from_millis(2500)).await;
                    if !window.is_visible().unwrap_or(true) {
                        let _ = window.show();
                    }
                });
            }

            if let Err(e) = tray::setup_tray(handle, &settings) {
                eprintln!("Failed to setup system tray: {e}");
            }

            if let Err(e) = shortcuts::register_global_shortcuts(handle, &settings) {
                eprintln!("Failed to register global shortcuts: {e}");
            }


            Ok(())
        })
        .on_window_event(|window, event| {
            if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                let app = window.app_handle();
                let state = app.state::<AppState>();
                // Keep running in the tray only while sound is playing; a paused
                // CACalm simply quits when its window is closed.
                let should_minimize = !QUITTING.load(Ordering::SeqCst)
                    && SOUND_PLAYING.load(Ordering::SeqCst)
                    && state
                        .settings
                        .lock()
                        .map(|s| s.minimize_to_tray && s.tray_enabled)
                        .unwrap_or(false);

                if should_minimize {
                    api.prevent_close();
                    let _ = window.hide();
                    if let Some(w) = app.get_webview_window(window.label()) {
                        memory::on_hidden(&w);
                    }
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
            commands::read_sound_file,
            commands::open_external,
            commands::set_sound_playing,
            commands::window_start_dragging,
            commands::window_minimize,
            commands::window_maximize,
            commands::window_close,
            commands::window_is_maximized
        ]);

    builder
        .run(tauri::generate_context!())
        .expect("error while running CACalm");
}
