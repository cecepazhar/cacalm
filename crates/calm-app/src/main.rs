mod commands;
mod lib;

use commands::get_timer_snapshot;

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_process::init())
        .manage(lib::AppState::new())
        .invoke_handler(tauri::generate_handler![get_timer_snapshot])
        .run(tauri::generate_context!())
        .expect("error while running CACalm");
}