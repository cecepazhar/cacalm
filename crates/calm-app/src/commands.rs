use crate::{settings::AppSettings, AppState};
use calm_core::{Preset, SoundEntry};
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::sync::Mutex;
use std::time::Duration;
use tauri::Emitter;

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
    use tauri::Manager;
    if let Some(win) = app.get_webview_window("main") {
        let _ = win.unminimize();
        let _ = win.show();
        let _ = win.set_focus();
    }
}

/// Toggle tray menu item text between "Start Focus" and "Pause Focus".
pub fn refresh_tray_toggle_label<R: tauri::Runtime>(app: &tauri::AppHandle<R>) {
    use tauri::Manager;
    let Some(tray) = app.tray_by_id(crate::tray::TRAY_ID) else {
        return;
    };
    let state: tauri::State<'_, AppState> = app.state();
    let Ok(timer) = state.timer.lock() else {
        return;
    };
    let label = if timer.is_paused() {
        "Start Focus"
    } else {
        "Pause Focus"
    };
    drop(timer);
    if let Some(item) = tray.menu().ok().and_then(|m| m.get(crate::tray::MENU_ITEM_TOGGLE_FOCUS)) {
        let _ = item.as_menuitem().map(|mi| mi.set_text(label));
    }
}

#[tauri::command]
pub fn get_timer_snapshot(state: tauri::State<'_, AppState>) -> Result<TimerSnapshot, String> {
    let timer = state.timer.lock().map_err(|e| e.to_string())?;
    Ok(snapshot(&timer))
}

#[tauri::command]
pub fn timer_pause(state: tauri::State<'_, AppState>) -> Result<TimerSnapshot, String> {
    let mut timer = state.timer.lock().map_err(|e| e.to_string())?;
    timer.pause();
    let snap = snapshot(&timer);
    crate::tray::update_tray_tooltip(state.as_ref(), &snap);
    Ok(snap)
}

#[tauri::command]
pub fn timer_resume(state: tauri::State<'_, AppState>) -> Result<TimerSnapshot, String> {
    let mut timer = state.timer.lock().map_err(|e| e.to_string())?;
    timer.resume();
    let snap = snapshot(&timer);
    crate::tray::update_tray_tooltip(state.as_ref(), &snap);
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
    let mut settings = state.settings.lock().map_err(|e| e.to_string())?;
    let was_enabled_shortcuts = settings.shortcuts_enabled;
    let was_enabled_tray = settings.tray_enabled;
    *settings = new_settings.clone();
    drop(settings);

    if was_enabled_shortcuts != settings.shortcuts_enabled
        || was_enabled_tray != settings.tray_enabled
        || settings.shortcut_timer_toggle
            != settings.shortcut_open_app
            && false
    {
        let _ = crate::shortcuts::unregister_global_shortcuts(&app);
        if settings.shortcuts_enabled {
            if let Err(e) = crate::shortcuts::register_global_shortcuts(&app, settings) {
                eprintln!("Shortcut re-registration failed: {e}");
            }
        }
    }

    crate::tray::set_tray_visible(&app, settings.tray_enabled);
    Ok(())
}

#[tauri::command]
pub fn toggle_tray(
    state: tauri::State<'_, AppState>,
    app: tauri::AppHandle,
) -> Result<AppSettings, String> {
    let mut settings = state.settings.lock().map_err(|e| e.to_string())?;
    settings.tray_enabled = !settings.tray_enabled;
    crate::settings::save_settings(settings)?;
    drop(settings);
    crate::tray::set_tray_visible(&app, settings.tray_enabled);
    Ok(settings.clone())
}

#[tauri::command]
pub fn toggle_shortcuts(
    state: tauri::State<'_, AppState>,
    app: tauri::AppHandle,
) -> Result<AppSettings, String> {
    let mut settings = state.settings.lock().map_err(|e| e.to_string())?;
    settings.shortcuts_enabled = !settings.shortcuts_enabled;
    let snapshot = settings.clone();
    crate::settings::save_settings(settings)?;
    drop(settings);

    let _ = crate::shortcuts::unregister_global_shortcuts(&app);
    if snapshot.shortcuts_enabled {
        if let Err(e) = crate::shortcuts::register_global_shortcuts(&app, &snapshot) {
            eprintln!("Shortcut registration failed: {e}");
        }
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
    crate::settings::save_settings(settings)?;
    drop(settings);

    let _ = crate::shortcuts::unregister_global_shortcuts(&app);
    if enabled {
        if let Err(e) = crate::shortcuts::register_global_shortcuts(&app, &snapshot) {
            eprintln!("Shortcut registration failed: {e}");
        }
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
    crate::settings::save_settings(settings)?;
    drop(settings);

    crate::tray::set_tray_visible(&app, enabled);
    Ok(snapshot)
}

/// Minimize the main window to the tray (used by window close handler and
/// frontend "minimize" button).
#[tauri::command]
pub fn minimize_to_tray(app: tauri::AppHandle) {
    use tauri::Manager;
    if let Some(win) = app.get_webview_window("main") {
        let _ = win.hide();
    }
}
