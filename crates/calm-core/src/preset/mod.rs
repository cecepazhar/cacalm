//! Preset manager — save/load mixes from JSON files.
//!
//! Presets are stored as JSON in `~/.local/share/cacalm/presets/*.json`.

use crate::paths::app_data_dir;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PresetChannel {
    pub channel_id: String,
    pub name: String,
    pub source_path: String,
    pub volume: f32,
    pub muted: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Preset {
    pub id: String,
    pub name: String,
    pub channels: Vec<PresetChannel>,
    pub master_volume: f32,
    pub created_at: i64,
}

#[derive(Debug, thiserror::Error)]
pub enum PresetError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("Preset not found: {0}")]
    NotFound(String),
}

pub struct PresetManager {
    presets_dir: PathBuf,
}

impl PresetManager {
    pub fn new() -> Result<Self, PresetError> {
        let data_dir = app_data_dir().map_err(PresetError::Io)?;
        let presets_dir = data_dir.join("presets");
        fs::create_dir_all(&presets_dir)?;
        Ok(Self { presets_dir })
    }

    pub fn save(&self, preset: &Preset) -> Result<(), PresetError> {
        let path = self.presets_dir.join(format!("{}.json", preset.id));
        let json = serde_json::to_string_pretty(preset)?;
        fs::write(path, json)?;
        Ok(())
    }

    pub fn load(&self, id: &str) -> Result<Preset, PresetError> {
        let path = self.presets_dir.join(format!("{}.json", id));
        let json = fs::read_to_string(path)?;
        Ok(serde_json::from_str(&json)?)
    }

    pub fn list(&self) -> Result<Vec<Preset>, PresetError> {
        let mut presets: Vec<Preset> = Vec::new();
        if let Ok(entries) = fs::read_dir(&self.presets_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().map_or(false, |ext| ext == "json") {
                    if let Ok(json) = fs::read_to_string(&path) {
                        if let Ok(preset) = serde_json::from_str(&json) {
                            presets.push(preset);
                        }
                    }
                }
            }
        }
        presets.sort_by_key(|p| p.created_at);
        presets.reverse();
        Ok(presets)
    }

    pub fn delete(&self, id: &str) -> Result<(), PresetError> {
        let path = self.presets_dir.join(format!("{}.json", id));
        if !path.exists() {
            return Err(PresetError::NotFound(id.to_string()));
        }
        fs::remove_file(path)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn save_and_load_roundtrip() {
        let tmp = std::env::temp_dir().join(format!(
            "cacalm-preset-test-{}",
            uuid::Uuid::now_v7()
        ));
        // Override app_data_dir by symlinking
        unsafe { std::env::set_var("HOME", &tmp) };

        let mgr = PresetManager::new().unwrap();
        let preset = Preset {
            id: "test".into(),
            name: "Test".into(),
            channels: vec![],
            master_volume: 0.8,
            created_at: 1000,
        };
        mgr.save(&preset).unwrap();
        let loaded = mgr.load("test").unwrap();
        assert_eq!(loaded.name, "Test");
        assert_eq!(loaded.master_volume, 0.8);
    }
}
