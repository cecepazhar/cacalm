//! WebView2 memory governance (same approach as CAPost).
//!
//! While CACalm is in the background — unfocused for a while, minimized, or hidden in
//! the tray — WebView2 is asked to drop caches and compact its heap
//! (`MemoryUsageTargetLevel::Low`). As soon as the window is focused again it goes back to
//! normal. Audio keeps playing either way; this only trims caches in the renderer and GPU
//! processes, which is where most of the memory lives.

/// Visible but unfocused for this long counts as "in the background".
#[cfg(windows)]
const BACKGROUND_AFTER: std::time::Duration = std::time::Duration::from_secs(20);

#[cfg(windows)]
fn set_low_memory(window: &tauri::WebviewWindow, low: bool) {
    let _ = window.with_webview(move |webview| {
        use webview2_com::Microsoft::Web::WebView2::Win32::{
            COREWEBVIEW2_MEMORY_USAGE_TARGET_LEVEL_LOW, COREWEBVIEW2_MEMORY_USAGE_TARGET_LEVEL_NORMAL,
            ICoreWebView2_19,
        };
        use windows_core::Interface;

        // SAFETY: plain COM calls on the live controller Tauri hands us, on the UI thread
        // that `with_webview` runs closures on (as WebView2 requires).
        unsafe {
            let Ok(core) = webview.controller().CoreWebView2() else {
                return;
            };
            // ICoreWebView2_19 needs WebView2 Runtime 1.0.2210+; older runtimes keep the default.
            let Ok(core19) = core.cast::<ICoreWebView2_19>() else {
                return;
            };
            let level = if low {
                COREWEBVIEW2_MEMORY_USAGE_TARGET_LEVEL_LOW
            } else {
                COREWEBVIEW2_MEMORY_USAGE_TARGET_LEVEL_NORMAL
            };
            let _ = core19.SetMemoryUsageTargetLevel(level);
        }
    });
}

/// Installs the focus/minimize watcher on the main window.
#[cfg(windows)]
pub fn install(window: &tauri::WebviewWindow) {
    use std::sync::Arc;
    use std::sync::atomic::{AtomicU64, Ordering};
    use tauri::WindowEvent;

    // Every focus change bumps the generation, so a pending "go low" timer from an
    // earlier blur is ignored once the user has come back.
    let generation = Arc::new(AtomicU64::new(0));
    let handle = window.clone();
    window.on_window_event(move |event| match event {
        WindowEvent::Focused(true) => {
            generation.fetch_add(1, Ordering::SeqCst);
            set_low_memory(&handle, false);
        }
        WindowEvent::Focused(false) => {
            let ticket = generation.fetch_add(1, Ordering::SeqCst) + 1;
            let (generation, window) = (generation.clone(), handle.clone());
            tauri::async_runtime::spawn(async move {
                tokio::time::sleep(BACKGROUND_AFTER).await;
                if generation.load(Ordering::SeqCst) == ticket {
                    set_low_memory(&window, true);
                }
            });
        }
        WindowEvent::Resized(_) if handle.is_minimized().unwrap_or(false) => {
            generation.fetch_add(1, Ordering::SeqCst);
            set_low_memory(&handle, true);
        }
        _ => {}
    });
}

/// Hidden to the tray: trim right away.
#[cfg(windows)]
pub fn on_hidden(window: &tauri::WebviewWindow) {
    set_low_memory(window, true);
}

#[cfg(not(windows))]
pub fn install(_window: &tauri::WebviewWindow) {}

#[cfg(not(windows))]
pub fn on_hidden(_window: &tauri::WebviewWindow) {}
