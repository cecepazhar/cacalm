use crate::commands::TimerSnapshot;
use crate::settings::AppSettings;
use tauri::{
    menu::MenuBuilder,
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    AppHandle, Manager, Runtime,
};

pub const TRAY_ID: &str = "cacalm-tray";
pub const MENU_ITEM_TOGGLE_FOCUS: &str = "toggle_focus";
pub const MENU_ITEM_OPEN_APP: &str = "open_app";
pub const MENU_ITEM_EXIT: &str = "exit";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TrayAction {
    ToggleFocus,
    OpenApp,
    Exit,
}

impl TrayAction {
    pub fn from_id(id: &str) -> Option<Self> {
        match id {
            MENU_ITEM_TOGGLE_FOCUS => Some(Self::ToggleFocus),
            MENU_ITEM_OPEN_APP => Some(Self::OpenApp),
            MENU_ITEM_EXIT => Some(Self::Exit),
            _ => None,
        }
    }
}

/// Pure state machine representing tray visibility and tooltip/menu state.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TrayState {
    pub visible: bool,
    pub timer_running: bool,
    pub current_phase: String,
    pub remaining_secs: u64,
}

impl TrayState {
    pub fn new(visible: bool) -> Self {
        Self {
            visible,
            timer_running: false,
            current_phase: "FOCUS".to_string(),
            remaining_secs: 25 * 60,
        }
    }

    pub fn update_from_snapshot(&mut self, phase: &str, remaining_secs: u64, paused: bool) {
        self.current_phase = phase.to_string();
        self.remaining_secs = remaining_secs;
        self.timer_running = !paused;
    }

    pub fn menu_toggle_label(&self) -> &'static str {
        "Start/Pause Focus"
    }

    pub fn dynamic_toggle_label(&self) -> &'static str {
        if self.timer_running {
            "Pause Focus"
        } else {
            "Start Focus"
        }
    }

    pub fn format_tooltip(&self) -> String {
        let mins = self.remaining_secs / 60;
        let secs = self.remaining_secs % 60;
        let status = if self.timer_running {
            "Running"
        } else {
            "Paused"
        };
        format!(
            "CACalm — {} {}:{:02} ({})",
            self.current_phase, mins, secs, status
        )
    }
}

pub fn setup_tray<R: Runtime>(
    app: &AppHandle<R>,
    settings: &AppSettings,
) -> Result<(), Box<dyn std::error::Error>> {
    let menu = MenuBuilder::new(app)
        .text(MENU_ITEM_TOGGLE_FOCUS, "Start/Pause Focus")
        .text(MENU_ITEM_OPEN_APP, "Open App")
        .separator()
        .text(MENU_ITEM_EXIT, "Exit")
        .build()?;

    let mut builder = TrayIconBuilder::with_id(TRAY_ID)
        .tooltip("CACalm")
        .menu(&menu)
        .show_menu_on_left_click(true);

    if let Some(icon) = app.default_window_icon() {
        builder = builder.icon(icon.clone());
    } else if let Ok(icon) = tauri::image::Image::from_bytes(include_bytes!("../icons/32x32.png")) {
        builder = builder.icon(icon);
    }

    builder
        .on_menu_event(|app, event| {
            if let Some(action) = TrayAction::from_id(event.id.as_ref()) {
                match action {
                    TrayAction::ToggleFocus => {
                        crate::toggle_timer_state(app);
                    }
                    TrayAction::OpenApp => {
                        crate::open_main_window(app);
                    }
                    TrayAction::Exit => {
                        app.exit(0);
                    }
                }
            }
        })
        .on_tray_icon_event(|tray, event| {
            if let TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                ..
            } = event
            {
                crate::open_main_window(tray.app_handle());
            }
        })
        .build(app)?;

    if !settings.tray_enabled {
        set_tray_visible(app, false);
    }

    Ok(())
}

pub fn set_tray_visible<R: Runtime>(app: &AppHandle<R>, visible: bool) {
    if let Some(tray) = app.tray_by_id(TRAY_ID) {
        let _ = tray.set_visible(visible);
    }
}

pub fn update_tray_tooltip<R: Runtime>(app: &AppHandle<R>, snapshot: &TimerSnapshot) {
    if let Some(tray) = app.tray_by_id(TRAY_ID) {
        let mut state = TrayState::new(true);
        state.update_from_snapshot(&snapshot.phase, snapshot.remaining, snapshot.paused);
        let _ = tray.set_tooltip(Some(state.format_tooltip()));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tray_action_from_id() {
        assert_eq!(
            TrayAction::from_id(MENU_ITEM_TOGGLE_FOCUS),
            Some(TrayAction::ToggleFocus)
        );
        assert_eq!(
            TrayAction::from_id(MENU_ITEM_OPEN_APP),
            Some(TrayAction::OpenApp)
        );
        assert_eq!(TrayAction::from_id(MENU_ITEM_EXIT), Some(TrayAction::Exit));
        assert_eq!(TrayAction::from_id("unknown_id"), None);
    }

    #[test]
    fn test_tray_state_labels_and_tooltip() {
        let mut state = TrayState::new(true);
        assert!(state.visible);
        assert_eq!(state.menu_toggle_label(), "Start/Pause Focus");
        assert_eq!(state.dynamic_toggle_label(), "Start Focus");

        state.update_from_snapshot("FOCUS", 25 * 60, false);
        assert!(state.timer_running);
        assert_eq!(state.dynamic_toggle_label(), "Pause Focus");
        assert_eq!(state.format_tooltip(), "CACalm — FOCUS 25:00 (Running)");

        state.update_from_snapshot("SHORT BREAK", 4 * 60 + 45, true);
        assert!(!state.timer_running);
        assert_eq!(state.dynamic_toggle_label(), "Start Focus");
        assert_eq!(
            state.format_tooltip(),
            "CACalm — SHORT BREAK 4:45 (Paused)"
        );
    }

    #[test]
    fn test_tray_state_visibility_toggle() {
        let mut state = TrayState::new(true);
        assert!(state.visible);
        state.visible = false;
        assert!(!state.visible);
    }
}
