use crate::AppState;

#[tauri::command]
fn get_timer_snapshot(state: tauri::State<'_, AppState>) -> Result<crate::TimerSnapshot, String> {
    crate::get_timer_snapshot(state)
}