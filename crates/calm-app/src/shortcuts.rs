use crate::settings::AppSettings;
use tauri::{AppHandle, Runtime};
use tauri_plugin_global_shortcut::{GlobalShortcutExt, ShortcutState};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShortcutAction {
    ToggleTimer,
    OpenApp,
}

/// Pure state machine representing active shortcuts configuration and matching.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ShortcutModel {
    pub timer_shortcut: String,
    pub open_app_shortcut: String,
    pub enabled: bool,
}

impl ShortcutModel {
    pub fn new(timer_shortcut: String, open_app_shortcut: String, enabled: bool) -> Self {
        Self {
            timer_shortcut,
            open_app_shortcut,
            enabled,
        }
    }

    pub fn resolve_action(&self, pressed: &str) -> Option<ShortcutAction> {
        if !self.enabled {
            return None;
        }
        if self.timer_shortcut.eq_ignore_ascii_case(pressed) {
            Some(ShortcutAction::ToggleTimer)
        } else if self.open_app_shortcut.eq_ignore_ascii_case(pressed) {
            Some(ShortcutAction::OpenApp)
        } else {
            None
        }
    }

    pub fn is_valid_shortcut(s: &str) -> bool {
        let parts: Vec<&str> = s.split('+').map(|p| p.trim()).collect();
        if parts.len() < 2 {
            return false;
        }
        let has_modifier = parts.iter().any(|&p| {
            p.eq_ignore_ascii_case("ctrl")
                || p.eq_ignore_ascii_case("control")
                || p.eq_ignore_ascii_case("alt")
                || p.eq_ignore_ascii_case("shift")
                || p.eq_ignore_ascii_case("cmd")
                || p.eq_ignore_ascii_case("cmdorcontrol")
                || p.eq_ignore_ascii_case("super")
        });
        let has_key = parts.iter().any(|&p| {
            !p.eq_ignore_ascii_case("ctrl")
                && !p.eq_ignore_ascii_case("control")
                && !p.eq_ignore_ascii_case("alt")
                && !p.eq_ignore_ascii_case("shift")
                && !p.eq_ignore_ascii_case("cmd")
                && !p.eq_ignore_ascii_case("cmdorcontrol")
                && !p.eq_ignore_ascii_case("super")
                && !p.is_empty()
        });
        has_modifier && has_key
    }
}

pub fn register_global_shortcuts<R: Runtime>(
    app: &AppHandle<R>,
    settings: &AppSettings,
) -> Result<(), Box<dyn std::error::Error>> {
    let global_shortcut = app.global_shortcut();
    let _ = global_shortcut.unregister_all();

    if !settings.shortcuts_enabled {
        return Ok(());
    }

    let timer_sc = settings.shortcut_timer_toggle.clone();
    let app_handle1 = app.clone();
    if let Err(e) = global_shortcut.on_shortcut(timer_sc.as_str(), move |_app, _sc, event| {
        if event.state == ShortcutState::Pressed {
            let _ = crate::toggle_timer_state(&app_handle1);
        }
    }) {
        eprintln!(
            "Warning: could not register timer shortcut {}: {}",
            timer_sc, e
        );
    }

    let open_sc = settings.shortcut_open_app.clone();
    let app_handle2 = app.clone();
    if let Err(e) = global_shortcut.on_shortcut(open_sc.as_str(), move |_app, _sc, event| {
        if event.state == ShortcutState::Pressed {
            crate::open_main_window(&app_handle2);
        }
    }) {
        eprintln!(
            "Warning: could not register open app shortcut {}: {}",
            open_sc, e
        );
    }

    Ok(())
}

pub fn unregister_global_shortcuts<R: Runtime>(app: &AppHandle<R>) {
    let _ = app.global_shortcut().unregister_all();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_shortcut_validation() {
        assert!(ShortcutModel::is_valid_shortcut("Ctrl+Alt+C"));
        assert!(ShortcutModel::is_valid_shortcut("CmdOrControl+Alt+L"));
        assert!(ShortcutModel::is_valid_shortcut("Shift+Alt+P"));
        assert!(ShortcutModel::is_valid_shortcut("ctrl+c"));

        assert!(!ShortcutModel::is_valid_shortcut(""));
        assert!(!ShortcutModel::is_valid_shortcut("C"));
        assert!(!ShortcutModel::is_valid_shortcut("Ctrl+"));
        assert!(!ShortcutModel::is_valid_shortcut("+Alt"));
        assert!(!ShortcutModel::is_valid_shortcut("Ctrl+Alt"));
    }

    #[test]
    fn test_resolve_action_when_enabled() {
        let model =
            ShortcutModel::new("Ctrl+Alt+C".to_string(), "Ctrl+Alt+L".to_string(), true);

        assert_eq!(
            model.resolve_action("Ctrl+Alt+C"),
            Some(ShortcutAction::ToggleTimer)
        );
        assert_eq!(
            model.resolve_action("ctrl+alt+c"),
            Some(ShortcutAction::ToggleTimer)
        );
        assert_eq!(
            model.resolve_action("Ctrl+Alt+L"),
            Some(ShortcutAction::OpenApp)
        );
        assert_eq!(
            model.resolve_action("ctrl+alt+l"),
            Some(ShortcutAction::OpenApp)
        );
        assert_eq!(model.resolve_action("Ctrl+Alt+X"), None);
    }

    #[test]
    fn test_resolve_action_when_disabled() {
        let model =
            ShortcutModel::new("Ctrl+Alt+C".to_string(), "Ctrl+Alt+L".to_string(), false);

        assert_eq!(model.resolve_action("Ctrl+Alt+C"), None);
        assert_eq!(model.resolve_action("Ctrl+Alt+L"), None);
    }
}
