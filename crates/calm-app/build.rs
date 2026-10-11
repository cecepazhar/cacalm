pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_process::init())
        .manage(App::new())
        .invoke_handler(tauri::generate_handler![get_timer_snapshot])
        .run(tauri::generate_context!())
        .expect("error while running CACalm");
}
