use crate::AppState;
use calm_core::{Preset, SoundEntry};
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::time::Duration;

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

#[tauri::command]
pub fn get_timer_snapshot(state: tauri::State<'_, AppState>) -> Result<TimerSnapshot, String> {
    let timer = state.timer.lock().map_err(|e| e.to_string())?;
    Ok(snapshot(&timer))
}

#[tauri::command]
pub fn timer_pause(state: tauri::State<'_, AppState>) -> Result<TimerSnapshot, String> {
    let mut timer = state.timer.lock().map_err(|e| e.to_string())?;
    timer.pause();
    Ok(snapshot(&timer))
}

#[tauri::command]
pub fn timer_resume(state: tauri::State<'_, AppState>) -> Result<TimerSnapshot, String> {
    let mut timer = state.timer.lock().map_err(|e| e.to_string())?;
    timer.resume();
    Ok(snapshot(&timer))
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
