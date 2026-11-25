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

// --- Immersive mode & end-of-timer actions ---

/// Full-screen on/off for Immersive Mode.
#[tauri::command]
pub async fn window_set_fullscreen(
    #[allow(unused_variables)] window: tauri::Window,
    #[allow(unused_variables)] fullscreen: bool,
) {
    #[cfg(desktop)]
    let _ = window.set_fullscreen(fullscreen);
}

/// Brings CACalm to the front (out of the tray / minimized) so the user can see and
/// cancel the power countdown.
#[tauri::command]
pub fn window_attention(app: tauri::AppHandle) {
    open_main_window(&app);
}

/// Quits CACalm completely (sleep-timer action "close CACalm").
#[tauri::command]
pub fn app_quit(app: tauri::AppHandle) {
    crate::quit_app(&app);
}

/// The only power actions the UI may request. Never a free-form command.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PowerAction {
    Sleep,
    Shutdown,
}

impl PowerAction {
    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "sleep" => Some(Self::Sleep),
            "shutdown" => Some(Self::Shutdown),
            _ => None,
        }
    }
}

fn power_command(action: PowerAction) -> std::process::Command {
    use std::process::Command;
    #[cfg(target_os = "windows")]
    {
        match action {
            // Application.SetSuspendState(Suspend) is a real sleep (rundll32 SetSuspendState
            // hibernates instead when hibernation is enabled).
            PowerAction::Sleep => {
                let mut c = Command::new("powershell");
                c.args([
                    "-NoProfile",
                    "-NonInteractive",
                    "-Command",
                    "Add-Type -AssemblyName System.Windows.Forms; [System.Windows.Forms.Application]::SetSuspendState('Suspend', $false, $false) | Out-Null",
                ]);
                c
            }
            PowerAction::Shutdown => {
                let mut c = Command::new("shutdown");
                c.args(["/s", "/t", "0"]);
                c
            }
        }
    }
    #[cfg(target_os = "macos")]
    {
        match action {
            PowerAction::Sleep => {
                let mut c = Command::new("pmset");
                c.arg("sleepnow");
                c
            }
            PowerAction::Shutdown => {
                let mut c = Command::new("osascript");
                c.args(["-e", "tell application \"System Events\" to shut down"]);
                c
            }
        }
    }
    #[cfg(all(unix, not(target_os = "macos")))]
    {
        let mut c = Command::new("systemctl");
        c.arg(match action {
            PowerAction::Sleep => "suspend",
            PowerAction::Shutdown => "poweroff",
        });
        c
    }
}

/// Puts the computer to sleep or shuts it down when the sleep timer ends.
/// The UI always shows a 60-second cancellable countdown before calling this.
#[tauri::command]
pub fn system_power(action: String) -> Result<(), String> {
    let action = PowerAction::parse(&action).ok_or_else(|| "unknown power action".to_string())?;
    #[cfg(target_os = "windows")]
    {
        use std::os::windows::process::CommandExt;
        const CREATE_NO_WINDOW: u32 = 0x0800_0000;
        power_command(action)
            .creation_flags(CREATE_NO_WINDOW)
            .spawn()
            .map(|_| ())
            .map_err(|e| e.to_string())
    }
    #[cfg(not(target_os = "windows"))]
    {
        power_command(action).spawn().map(|_| ()).map_err(|e| e.to_string())
    }
}

// --- Backup & restore ---

const BACKUP_MAX_BYTES: u64 = 2 * 1024 * 1024;

fn check_backup_path(path: &str) -> Result<std::path::PathBuf, String> {
    let p = std::path::PathBuf::from(path);
    let is_json = p
        .extension()
        .and_then(|e| e.to_str())
        .is_some_and(|e| e.eq_ignore_ascii_case("json"));
    if !is_json {
        return Err("backup must be a .json file".into());
    }
    Ok(p)
}

/// Writes a configuration backup chosen by the user in the save dialog.
#[tauri::command]
pub fn backup_write(path: String, contents: String) -> Result<(), String> {
    let p = check_backup_path(&path)?;
    if contents.len() as u64 > BACKUP_MAX_BYTES {
        return Err("backup too large".into());
    }
    std::fs::write(p, contents).map_err(|e| e.to_string())
}

/// Reads a configuration backup chosen by the user in the open dialog.
#[tauri::command]
pub fn backup_read(path: String) -> Result<String, String> {
    let p = check_backup_path(&path)?;
    let len = std::fs::metadata(&p).map_err(|e| e.to_string())?.len();
    if len > BACKUP_MAX_BYTES {
        return Err("backup too large".into());
    }
    std::fs::read_to_string(p).map_err(|e| e.to_string())
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
    fn backups_must_be_small_json_files() {
        let dir = std::env::temp_dir().join(format!("cacalm-backup-test-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let ok = dir.join("b.json");
        super::backup_write(ok.to_string_lossy().into(), "{\"a\":1}".into()).unwrap();
        assert_eq!(super::backup_read(ok.to_string_lossy().into()).unwrap(), "{\"a\":1}");
        assert!(super::backup_write(dir.join("evil.exe").to_string_lossy().into(), "x".into()).is_err());
        assert!(super::backup_read(dir.join("x.txt").to_string_lossy().into()).is_err());
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn power_actions_are_allow_listed() {
        use super::PowerAction;
        assert_eq!(PowerAction::parse("sleep"), Some(PowerAction::Sleep));
        assert_eq!(PowerAction::parse("shutdown"), Some(PowerAction::Shutdown));
        assert_eq!(PowerAction::parse("rm -rf /"), None);
        assert!(super::system_power("reboot".into()).is_err());
    }

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
