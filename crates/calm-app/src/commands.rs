use crate::{settings::AppSettings, AppState};
use calm_core::{Preset, ProStatus, SoundEntry};
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::time::Duration;
use tauri::{Emitter, Manager};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct TimerSnapshot {
    pub phase: String,
    pub remaining: u64,
    pub paused: bool,
    pub sessions_today: u32,
    pub focus_seconds_today: u64,
}

fn snapshot(timer: &calm_core::PomodoroTimer) -> TimerSnapshot {
    TimerSnapshot {
        phase: timer.phase().label().to_string(),
        remaining: timer.remaining().as_secs(),
        paused: timer.is_paused(),
        sessions_today: timer.sessions_today,
        focus_seconds_today: timer.focus_seconds_today,
    }
}

/// Toggle timer between paused and running; used by the tray menu and global
/// shortcut. Emits `timer-updated` so open windows stay in sync.
pub fn toggle_timer_state<R: tauri::Runtime>(app: &tauri::AppHandle<R>) -> Result<(), String> {
    let state: tauri::State<'_, AppState> = app.state();
    let mut timer = state.timer.lock().map_err(|e| e.to_string())?;
    if timer.is_paused() {
        timer.resume();
    } else {
        timer.pause();
    }
    let snap = snapshot(&timer);
    drop(timer);

    crate::tray::update_tray_tooltip(app, &snap);
    let _ = app.emit("timer-updated", snap);
    Ok(())
}

/// Show and focus the main window (used by tray "Open App" / shortcut).
pub fn open_main_window<R: tauri::Runtime>(app: &tauri::AppHandle<R>) {
    if let Some(win) = app.get_webview_window("main") {
        let _ = win.unminimize();
        let _ = win.show();
        let _ = win.set_focus();
    }
}

#[tauri::command]
pub fn get_timer_snapshot(state: tauri::State<'_, AppState>) -> Result<TimerSnapshot, String> {
    let timer = state.timer.lock().map_err(|e| e.to_string())?;
    Ok(snapshot(&timer))
}

#[tauri::command]
pub fn timer_pause(
    app: tauri::AppHandle,
    state: tauri::State<'_, AppState>,
) -> Result<TimerSnapshot, String> {
    let mut timer = state.timer.lock().map_err(|e| e.to_string())?;
    timer.pause();
    let snap = snapshot(&timer);
    crate::tray::update_tray_tooltip(&app, &snap);
    Ok(snap)
}

#[tauri::command]
pub fn timer_resume(
    app: tauri::AppHandle,
    state: tauri::State<'_, AppState>,
) -> Result<TimerSnapshot, String> {
    let mut timer = state.timer.lock().map_err(|e| e.to_string())?;
    timer.resume();
    let snap = snapshot(&timer);
    crate::tray::update_tray_tooltip(&app, &snap);
    Ok(snap)
}

#[tauri::command]
pub fn timer_reset(state: tauri::State<'_, AppState>) -> Result<TimerSnapshot, String> {
    let mut timer = state.timer.lock().map_err(|e| e.to_string())?;
    timer.reset();
    Ok(snapshot(&timer))
}

#[tauri::command]
pub fn timer_skip(state: tauri::State<'_, AppState>) -> Result<TimerSnapshot, String> {
    let mut timer = state.timer.lock().map_err(|e| e.to_string())?;
    timer.skip();
    Ok(snapshot(&timer))
}

#[tauri::command]
pub fn timer_tick(
    state: tauri::State<'_, AppState>,
    elapsed_secs: u64,
) -> Result<TimerSnapshot, String> {
    let mut timer = state.timer.lock().map_err(|e| e.to_string())?;
    timer.tick(Duration::from_secs(elapsed_secs));
    Ok(snapshot(&timer))
}

#[tauri::command]
pub fn list_sounds(state: tauri::State<'_, AppState>) -> Result<Vec<SoundEntry>, String> {
    let lib = state.library.as_ref().map_err(|e| e.clone())?;
    lib.list().map_err(|e| e.to_string())
}

#[tauri::command]
pub fn import_sound(
    state: tauri::State<'_, AppState>,
    source_path: String,
    name: String,
    category: String,
) -> Result<SoundEntry, String> {
    let lib = state.library.as_ref().map_err(|e| e.clone())?;
    lib.import(Path::new(&source_path), &name, &category)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn delete_sound(state: tauri::State<'_, AppState>, id: String) -> Result<(), String> {
    let lib = state.library.as_ref().map_err(|e| e.clone())?;
    lib.delete(&id).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn list_presets(state: tauri::State<'_, AppState>) -> Result<Vec<Preset>, String> {
    let mgr = state.preset_mgr.as_ref().map_err(|e| e.clone())?;
    mgr.list().map_err(|e| e.to_string())
}

#[tauri::command]
pub fn save_preset(state: tauri::State<'_, AppState>, preset: Preset) -> Result<(), String> {
    let mgr = state.preset_mgr.as_ref().map_err(|e| e.clone())?;
    mgr.save(&preset).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn load_preset(state: tauri::State<'_, AppState>, id: String) -> Result<Preset, String> {
    let mgr = state.preset_mgr.as_ref().map_err(|e| e.clone())?;
    mgr.load(&id).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn delete_preset(state: tauri::State<'_, AppState>, id: String) -> Result<(), String> {
    let mgr = state.preset_mgr.as_ref().map_err(|e| e.clone())?;
    mgr.delete(&id).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_app_version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}

// --- Settings / Tray / Shortcuts (F6 & F7) ---

#[tauri::command]
pub fn get_settings(state: tauri::State<'_, AppState>) -> Result<AppSettings, String> {
    let settings = state.settings.lock().map_err(|e| e.to_string())?;
    Ok(settings.clone())
}

#[tauri::command]
pub fn update_settings(
    state: tauri::State<'_, AppState>,
    app: tauri::AppHandle,
    new_settings: AppSettings,
) -> Result<(), String> {
    crate::settings::save_settings(&new_settings)?;
    {
        let mut settings = state.settings.lock().map_err(|e| e.to_string())?;
        *settings = new_settings.clone();
    }

    if new_settings.shortcuts_enabled {
        crate::shortcuts::unregister_global_shortcuts(&app);
        if let Err(e) =
            crate::shortcuts::register_global_shortcuts(&app, &new_settings)
        {
            eprintln!("Shortcut re-registration failed: {e}");
        }
    } else {
        crate::shortcuts::unregister_global_shortcuts(&app);
    }

    crate::tray::set_tray_visible(&app, new_settings.tray_enabled);
    Ok(())
}

#[tauri::command]
pub fn toggle_tray(
    state: tauri::State<'_, AppState>,
    app: tauri::AppHandle,
) -> Result<AppSettings, String> {
    let mut settings = state.settings.lock().map_err(|e| e.to_string())?;
    settings.tray_enabled = !settings.tray_enabled;
    let snapshot = settings.clone();
    crate::settings::save_settings(&snapshot)?;
    drop(settings);
    crate::tray::set_tray_visible(&app, snapshot.tray_enabled);
    Ok(snapshot)
}

#[tauri::command]
pub fn toggle_shortcuts(
    state: tauri::State<'_, AppState>,
    app: tauri::AppHandle,
) -> Result<AppSettings, String> {
    let mut settings = state.settings.lock().map_err(|e| e.to_string())?;
    settings.shortcuts_enabled = !settings.shortcuts_enabled;
    let snapshot = settings.clone();
    crate::settings::save_settings(&snapshot)?;
    drop(settings);

    crate::shortcuts::unregister_global_shortcuts(&app);
    if snapshot.shortcuts_enabled
        && let Err(e) = crate::shortcuts::register_global_shortcuts(&app, &snapshot)
    {
        eprintln!("Shortcut registration failed: {e}");
    }
    Ok(snapshot)
}

#[tauri::command]
pub fn set_shortcuts_enabled(
    state: tauri::State<'_, AppState>,
    app: tauri::AppHandle,
    enabled: bool,
) -> Result<AppSettings, String> {
    let mut settings = state.settings.lock().map_err(|e| e.to_string())?;
    settings.shortcuts_enabled = enabled;
    let snapshot = settings.clone();
    crate::settings::save_settings(&snapshot)?;
    drop(settings);

    crate::shortcuts::unregister_global_shortcuts(&app);
    if enabled
        && let Err(e) = crate::shortcuts::register_global_shortcuts(&app, &snapshot)
    {
        eprintln!("Shortcut registration failed: {e}");
    }
    Ok(snapshot)
}

#[tauri::command]
pub fn set_tray_enabled(
    state: tauri::State<'_, AppState>,
    app: tauri::AppHandle,
    enabled: bool,
) -> Result<AppSettings, String> {
    let mut settings = state.settings.lock().map_err(|e| e.to_string())?;
    settings.tray_enabled = enabled;
    let snapshot = settings.clone();
    crate::settings::save_settings(&snapshot)?;
    drop(settings);

    crate::tray::set_tray_visible(&app, enabled);
    Ok(snapshot)
}

/// Minimize the main window to the tray (used by window close handler and
/// frontend "minimize" button).
#[tauri::command]
pub fn minimize_to_tray(app: tauri::AppHandle) {
    if let Some(win) = app.get_webview_window("main") {
        let _ = win.hide();
    }
}

#[tauri::command]
pub async fn window_start_dragging(#[allow(unused_variables)] window: tauri::Window) {
    #[cfg(desktop)]
    let _ = window.start_dragging();
}

#[tauri::command]
pub async fn window_minimize(#[allow(unused_variables)] window: tauri::Window) {
    #[cfg(desktop)]
    let _ = window.minimize();
}

#[tauri::command]
pub async fn window_maximize(#[allow(unused_variables)] window: tauri::Window) {
    #[cfg(desktop)]
    {
        if let Ok(max) = window.is_maximized() {
            if max {
                let _ = window.unmaximize();
            } else {
                let _ = window.maximize();
            }
        } else {
            let _ = window.maximize();
        }
    }
}

#[tauri::command]
pub async fn window_close(#[allow(unused_variables)] window: tauri::Window) {
    #[cfg(desktop)]
    let _ = window.close();
}

#[tauri::command]
pub async fn window_is_maximized(#[allow(unused_variables)] window: tauri::Window) -> bool {
    #[cfg(desktop)]
    {
        window.is_maximized().unwrap_or(false)
    }
    #[cfg(not(desktop))]
    false
}

// --- Audio Engine IPC Commands ---

fn resolve_channel_audio_path(
    channel_id: &str,
    lib: Option<&calm_core::SoundLibrary>,
) -> Option<String> {
    // 1. Direct file path check
    let direct_path = std::path::Path::new(channel_id);
    if direct_path.is_file() {
        return Some(direct_path.to_string_lossy().to_string());
    }

    // 2. Default channel ID to file name mapping
    let default_filename = match channel_id {
        "ch-rain" | "rain" => Some("rain.wav"),
        "ch-waves" | "waves" | "ocean waves" => Some("waves.wav"),
        "ch-birds" | "birds" | "forest birds" => Some("birds.wav"),
        "ch-cafe" | "cafe" | "coffee shop" => Some("cafe.wav"),
        "ch-fire" | "fire" | "fireplace" => Some("fire.wav"),
        "ch-wind" | "wind" => Some("wind.wav"),
        "ch-stream" | "stream" => Some("stream.wav"),
        "ch-noise" | "noise" | "brown noise" => Some("noise.wav"),
        _ => None,
    };

    if let Some(lib) = lib {
        let sounds_dir = lib.sounds_dir();

        if let Some(fname) = default_filename {
            let p = sounds_dir.join(fname);
            if p.is_file() {
                return Some(p.to_string_lossy().to_string());
            }
        }

        let p_exact = sounds_dir.join(channel_id);
        if p_exact.is_file() {
            return Some(p_exact.to_string_lossy().to_string());
        }

        let p_wav = sounds_dir.join(format!("{channel_id}.wav"));
        if p_wav.is_file() {
            return Some(p_wav.to_string_lossy().to_string());
        }

        if let Ok(Some(entry)) = lib.get_sound_by_id(channel_id) {
            let p = lib.path_for(&entry);
            if p.is_file() {
                return Some(p.to_string_lossy().to_string());
            }
        }
    } else if let Ok(data_dir) = calm_core::paths::app_data_dir() {
        let sounds_dir = data_dir.join("sounds");
        if let Some(fname) = default_filename {
            let p = sounds_dir.join(fname);
            if p.is_file() {
                return Some(p.to_string_lossy().to_string());
            }
        }
    }

    None
}

#[tauri::command]
pub fn play_channel(
    state: tauri::State<'_, AppState>,
    channel_id: String,
    volume: f32,
) -> Result<(), String> {
    let engine = state.engine.as_ref().map_err(|e| e.clone())?;
    let ch_id = calm_core::ChannelId::from_string(&channel_id);
    let lib = state.library.as_ref().ok();
    let source_path = resolve_channel_audio_path(&channel_id, lib).unwrap_or_default();
    engine
        .play_channel(&ch_id, &channel_id, &source_path, volume)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn stop_channel(
    state: tauri::State<'_, AppState>,
    channel_id: String,
) -> Result<(), String> {
    let engine = state.engine.as_ref().map_err(|e| e.clone())?;
    let ch_id = calm_core::ChannelId::from_string(channel_id);
    engine.stop_channel(&ch_id).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn set_channel_volume(
    state: tauri::State<'_, AppState>,
    channel_id: String,
    volume: f32,
) -> Result<(), String> {
    let engine = state.engine.as_ref().map_err(|e| e.clone())?;
    let ch_id = calm_core::ChannelId::from_string(channel_id);
    engine.set_channel_volume(&ch_id, volume);
    Ok(())
}

#[tauri::command]
pub fn set_master_volume(
    state: tauri::State<'_, AppState>,
    volume: f32,
) -> Result<(), String> {
    let engine = state.engine.as_ref().map_err(|e| e.clone())?;
    engine.set_master_volume(volume);
    Ok(())
}

#[tauri::command]
pub fn set_channel_pan(
    state: tauri::State<'_, AppState>,
    channel_id: String,
    pan: f32,
) -> Result<(), String> {
    let engine = state.engine.as_ref().map_err(|e| e.clone())?;
    let ch_id = calm_core::ChannelId::from_string(channel_id);
    engine.set_channel_pan(&ch_id, pan);
    Ok(())
}

#[tauri::command]
pub fn list_active_channels(
    state: tauri::State<'_, AppState>,
) -> Result<Vec<String>, String> {
    let engine = state.engine.as_ref().map_err(|e| e.clone())?;
    Ok(engine.list_active_channel_ids())
}

// --- Pro Licensing (Free vs Pro) ---

#[tauri::command]
pub fn get_pro_status() -> Result<ProStatus, String> {
    Ok(calm_core::get_pro_status())
}

#[tauri::command]
pub fn activate_pro_license(license_key: String) -> Result<ProStatus, String> {
    calm_core::activate_license(&license_key)
}

#[tauri::command]
pub fn deactivate_pro_license() -> Result<(), String> {
    calm_core::deactivate_license()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pro_commands_contract() {
        // Querying pro status returns a valid Result
        let status = get_pro_status().expect("get_pro_status should return Ok");
        // By default on clean state, plan should be string and is_pro is boolean
        assert!(!status.plan.is_empty());

        // Activating an invalid key returns an Err
        let err_res = activate_pro_license("INVALID-KEY-12345".to_string());
        assert!(err_res.is_err());

        // Deactivating returns Ok(())
        let deact_res = deactivate_pro_license();
        assert!(deact_res.is_ok());
    }

    #[test]
    fn test_resolve_channel_audio_path_mapping() {
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let tmp = std::env::temp_dir().join(format!("cacalm-resolve-test-{nanos}"));
        std::fs::create_dir_all(&tmp).unwrap();

        let lib = calm_core::SoundLibrary::open_at(tmp.clone()).unwrap();
        // ensure_default_sounds is called in open_at, so rain.wav exists
        let rain_path = resolve_channel_audio_path("ch-rain", Some(&lib));
        assert!(rain_path.is_some(), "ch-rain should resolve to rain.wav");
        assert!(rain_path.unwrap().ends_with("rain.wav"));

        let waves_path = resolve_channel_audio_path("ocean waves", Some(&lib));
        assert!(waves_path.is_some());
        assert!(waves_path.unwrap().ends_with("waves.wav"));

        let unknown = resolve_channel_audio_path("unknown-sound-xyz", Some(&lib));
        assert!(unknown.is_none());

        let _ = std::fs::remove_dir_all(&tmp);
    }

    #[test]
    fn test_audio_engine_channel_operations() {
        if let Ok(engine) = calm_core::AudioEngine::new() {
            let nanos = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos();
            let tmp = std::env::temp_dir().join(format!("cacalm-cmd-test-{nanos}"));
            let default_sounds = calm_core::audio::synth::ensure_default_sounds(&tmp).unwrap();
            let (_, path) = &default_sounds[0];

            let ch = calm_core::ChannelId::from("ch-rain");
            assert!(engine.play_channel(&ch, "Rain", path.to_str().unwrap(), 0.7).is_ok());
            assert_eq!(engine.channel_count(), 1);

            engine.set_channel_volume(&ch, 0.4);
            engine.set_channel_pan(&ch, 0.8);
            engine.set_master_volume(0.9);

            let active = engine.list_active_channel_ids();
            assert_eq!(active, vec!["ch-rain".to_string()]);

            assert!(engine.stop_channel(&ch).is_ok());
            assert_eq!(engine.channel_count(), 0);

            let _ = std::fs::remove_dir_all(&tmp);
        }
    }
}
