use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AppSettings {
    #[serde(default = "default_true")]
    pub tray_enabled: bool,
    #[serde(default = "default_true")]
    pub shortcuts_enabled: bool,
    #[serde(default = "default_true")]
    pub minimize_to_tray: bool,
    #[serde(default = "default_shortcut_timer")]
    pub shortcut_timer_toggle: String,
    #[serde(default = "default_shortcut_open")]
    pub shortcut_open_app: String,
}

fn default_true() -> bool {
    true
}

fn default_shortcut_timer() -> String {
    "Ctrl+Alt+C".to_string()
}

fn default_shortcut_open() -> String {
    "Ctrl+Alt+L".to_string()
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            tray_enabled: true,
            shortcuts_enabled: true,
            minimize_to_tray: true,
            shortcut_timer_toggle: default_shortcut_timer(),
            shortcut_open_app: default_shortcut_open(),
        }
    }
}

pub fn settings_file_path() -> Result<PathBuf, std::io::Error> {
    let data_dir = calm_core::paths::app_data_dir()?;
    Ok(data_dir.join("settings.json"))
}

pub fn load_settings() -> AppSettings {
    let Ok(path) = settings_file_path() else {
        return AppSettings::default();
    };

    if !path.exists() {
        return AppSettings::default();
    }

    fs::read_to_string(&path)
        .ok()
        .and_then(|content| serde_json::from_str(&content).ok())
        .unwrap_or_default()
}

pub fn save_settings(settings: &AppSettings) -> Result<(), String> {
    let path = settings_file_path().map_err(|e| e.to_string())?;
    let content = serde_json::to_string_pretty(settings).map_err(|e| e.to_string())?;
    fs::write(&path, content).map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_settings() {
        let s = AppSettings::default();
        assert!(s.tray_enabled);
        assert!(s.shortcuts_enabled);
        assert!(s.minimize_to_tray);
        assert_eq!(s.shortcut_timer_toggle, "Ctrl+Alt+C");
        assert_eq!(s.shortcut_open_app, "Ctrl+Alt+L");
    }

    #[test]
    fn test_json_serialization_roundtrip() {
        let mut s = AppSettings::default();
        s.tray_enabled = false;
        s.shortcut_timer_toggle = "Ctrl+Shift+T".to_string();

        let json = serde_json::to_string(&s).expect("serialize should succeed");
        let parsed: AppSettings = serde_json::from_str(&json).expect("deserialize should succeed");

        assert_eq!(s, parsed);
        assert!(!parsed.tray_enabled);
        assert_eq!(parsed.shortcut_timer_toggle, "Ctrl+Shift+T");
    }

    #[test]
    fn test_deserialize_partial_json() {
        let json = r#"{"tray_enabled": false}"#;
        let parsed: AppSettings = serde_json::from_str(json).expect("should fill defaults");
        assert!(!parsed.tray_enabled);
        assert!(parsed.shortcuts_enabled);
        assert!(parsed.minimize_to_tray);
        assert_eq!(parsed.shortcut_timer_toggle, "Ctrl+Alt+C");
    }
}
