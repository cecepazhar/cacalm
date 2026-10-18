use crate::AppState;
use serde::Serialize;

#[derive(Serialize, Clone)]
pub struct TimerSnapshot {
    pub phase: String,
    pub remaining: u64,
    pub paused: bool,
    pub sessions_today: u32,
}

/// Get current timer state (placeholder — will be wired to real timer in T3)
#[tauri::command]
pub fn get_timer_snapshot(state: tauri::State<'_, AppState>) -> Result<TimerSnapshot, String> {
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
