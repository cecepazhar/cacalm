use crate::{settings::AppSettings, AppState};
use calm_core::{Preset, SoundEntry};
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

// --- Custom Sound Files ---

/// Returns the raw bytes of an imported sound so the WebView can decode it
/// with the Web Audio API. Sent as a binary IPC response (no JSON overhead).
#[tauri::command]
pub fn read_sound_file(
    state: tauri::State<'_, AppState>,
    id: String,
) -> Result<tauri::ipc::Response, String> {
    let lib = state.library.as_ref().map_err(|e| e.clone())?;
    let entry = lib
        .get_sound_by_id(&id)
        .map_err(|e| e.to_string())?
        .ok_or_else(|| format!("sound not found: {id}"))?;
    let bytes = std::fs::read(lib.path_for(&entry)).map_err(|e| e.to_string())?;
    Ok(tauri::ipc::Response::new(bytes))
}

// --- Playback state ---

/// The UI reports whether sound is playing so closing the window can either
/// keep playing in the tray (playing) or quit cleanly (paused).
#[tauri::command]
pub fn set_sound_playing(playing: bool) {
    crate::SOUND_PLAYING.store(playing, std::sync::atomic::Ordering::SeqCst);
}

// --- External links ---

/// Only links owned by the author may be opened from the UI.
const ALLOWED_URLS: &[&str] = &[
    "https://github.com/cecepazhar",
    "https://github.com/sponsors/cecepazhar",
];

fn is_allowed_url(url: &str) -> bool {
    let owned = ALLOWED_URLS.iter().any(|p| {
        url == *p || url.strip_prefix(p).is_some_and(|rest| rest.starts_with('/'))
    });
    owned
        && url
            .chars()
            .all(|c| c.is_ascii_graphic() && !matches!(c, '"' | '\\' | '<' | '>' | '^' | '|' | '\'' | '&' | '%'))
}

/// Opens an allow-listed URL in the system browser.
#[tauri::command]
pub fn open_external(url: String) -> Result<(), String> {
    if !is_allowed_url(&url) {
        return Err("url not allowed".into());
    }
    #[cfg(target_os = "windows")]
    let res = std::process::Command::new("rundll32")
        .args(["url.dll,FileProtocolHandler", &url])
        .spawn();
    #[cfg(target_os = "macos")]
    let res = std::process::Command::new("open").arg(&url).spawn();
    #[cfg(all(unix, not(target_os = "macos")))]
    let res = std::process::Command::new("xdg-open").arg(&url).spawn();
    res.map(|_| ()).map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    #[test]
    fn only_author_urls_are_allowed() {
        assert!(super::is_allowed_url("https://github.com/cecepazhar/cacalm"));
        assert!(!super::is_allowed_url("https://evil.example.com"));
        assert!(!super::is_allowed_url("https://github.com/cecepazharX/evil"));
        assert!(!super::is_allowed_url("https://github.com/cecepazhar/x\"&calc"));
    }

    #[test]
    fn app_version_matches_package() {
        assert_eq!(super::get_app_version(), env!("CARGO_PKG_VERSION"));
    }
}
