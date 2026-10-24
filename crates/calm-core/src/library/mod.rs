//! Local sound library: import audio files, persist metadata, list/delete.
//!
//! Design notes (PRD §F3):
//! - Imported files are **copied** into the app data dir, never linked, so a
//!   moved/deleted source file cannot break the library.
//! - Metadata lives in SQLite. WAL mode is enabled for concurrent reads.

use crate::paths::app_data_dir;
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SoundEntry {
    pub id: String,
    pub name: String,
    pub category: String,
    pub duration_secs: f64,
    pub file_name: String,
    pub imported_at: i64,
}

#[derive(Debug, thiserror::Error)]
pub enum LibraryError {
    #[error("storage error: {0}")]
    Db(String),
    #[error("file error: {0}")]
    Io(String),
}

pub struct SoundLibrary {
    conn: Mutex<Connection>,
    sounds_dir: PathBuf,
}

impl SoundLibrary {
    /// Opens (creating if needed) the library database and sounds directory.
    pub fn open() -> Result<Self, LibraryError> {
        let data_dir = app_data_dir().map_err(|e| LibraryError::Io(e.to_string()))?;
        Self::open_at(data_dir)
    }

    /// Opens the library at a specific data directory (clean for unit tests).
    pub fn open_at(data_dir: PathBuf) -> Result<Self, LibraryError> {
        let sounds_dir = data_dir.join("sounds");
        fs::create_dir_all(&sounds_dir).map_err(|e| LibraryError::Io(e.to_string()))?;

        // Ensure default synthetic sounds exist on disk
        let _ = crate::audio::synth::ensure_default_sounds(&sounds_dir);

        let conn = Connection::open(data_dir.join("library.db"))
            .map_err(|e| LibraryError::Db(e.to_string()))?;
        conn.pragma_update(None, "journal_mode", "WAL")
            .map_err(|e| LibraryError::Db(e.to_string()))?;
        conn.execute_batch(
            r#"
            CREATE TABLE IF NOT EXISTS sounds (
                id            TEXT PRIMARY KEY,
                name          TEXT NOT NULL,
                category      TEXT NOT NULL,
                duration_secs REAL NOT NULL DEFAULT 0,
                file_name     TEXT NOT NULL,
                imported_at   INTEGER NOT NULL
            );
            "#,
        )
        .map_err(|e| LibraryError::Db(e.to_string()))?;

        Ok(Self {
            conn: Mutex::new(conn),
            sounds_dir,
        })
    }

    /// Access the sounds directory path.
    pub fn sounds_dir(&self) -> &Path {
        &self.sounds_dir
    }

    /// Seeds default sounds into the SQLite database if they are not already present.
    pub fn seed_default_sounds(&self) -> Result<(), LibraryError> {
        let _ = crate::audio::synth::ensure_default_sounds(&self.sounds_dir)
            .map_err(|e| LibraryError::Io(e.to_string()))?;
        let conn = self.conn.lock().map_err(|e| LibraryError::Db(e.to_string()))?;
        for def in crate::audio::synth::DEFAULT_SOUNDS {
            let count: i64 = conn
                .query_row(
                    "SELECT COUNT(*) FROM sounds WHERE file_name = ?1 OR id = ?2",
                    params![def.file_name, def.id],
                    |r| r.get(0),
                )
                .unwrap_or(0);
            if count == 0 {
                let _ = conn.execute(
                    "INSERT INTO sounds (id, name, category, duration_secs, file_name, imported_at)
                     VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                    params![
                        def.id,
                        def.name,
                        def.category,
                        def.duration_secs as f64,
                        def.file_name,
                        chrono::Utc::now().timestamp(),
                    ],
                );
            }
        }
        Ok(())
    }

    /// Retrieve an entry by id or name.
    pub fn get_sound_by_id(&self, id: &str) -> Result<Option<SoundEntry>, LibraryError> {
        let conn = self.conn.lock().map_err(|e| LibraryError::Db(e.to_string()))?;
        let mut stmt = conn
            .prepare(
                "SELECT id, name, category, duration_secs, file_name, imported_at
                 FROM sounds WHERE id = ?1 OR name = ?1 LIMIT 1",
            )
            .map_err(|e| LibraryError::Db(e.to_string()))?;

        let mut rows = stmt
            .query_map(params![id], |row| {
                Ok(SoundEntry {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    category: row.get(2)?,
                    duration_secs: row.get(3)?,
                    file_name: row.get(4)?,
                    imported_at: row.get(5)?,
                })
            })
            .map_err(|e| LibraryError::Db(e.to_string()))?;

        if let Some(r) = rows.next() {
            Ok(Some(r.map_err(|e| LibraryError::Db(e.to_string()))?))
        } else {
            Ok(None)
        }
    }

    /// Copy `source` into the library and record it. Returns the new entry.
    pub fn import(
        &self,
        source: &Path,
        name: &str,
        category: &str,
    ) -> Result<SoundEntry, LibraryError> {
        let file_name = source
            .file_name()
            .ok_or_else(|| LibraryError::Io("source has no file name".into()))?
            .to_string_lossy()
            .to_string();

        let dest = self.sounds_dir.join(&file_name);
        fs::copy(source, &dest).map_err(|e| LibraryError::Io(e.to_string()))?;

        let entry = SoundEntry {
            id: uuid::Uuid::now_v7().to_string(),
            name: name.to_string(),
            category: category.to_string(),
            duration_secs: 0.0,
            file_name,
            imported_at: chrono::Utc::now().timestamp(),
        };

        let conn = self.conn.lock().map_err(|e| LibraryError::Db(e.to_string()))?;
        conn.execute(
            "INSERT INTO sounds (id, name, category, duration_secs, file_name, imported_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![
                entry.id,
                entry.name,
                entry.category,
                entry.duration_secs,
                entry.file_name,
                entry.imported_at
            ],
        )
        .map_err(|e| LibraryError::Db(e.to_string()))?;

        Ok(entry)
    }

    pub fn list(&self) -> Result<Vec<SoundEntry>, LibraryError> {
        let conn = self.conn.lock().map_err(|e| LibraryError::Db(e.to_string()))?;
        let mut stmt = conn
            .prepare(
                "SELECT id, name, category, duration_secs, file_name, imported_at
                 FROM sounds ORDER BY imported_at DESC",
            )
            .map_err(|e| LibraryError::Db(e.to_string()))?;

        let rows = stmt
            .query_map([], |row| {
                Ok(SoundEntry {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    category: row.get(2)?,
                    duration_secs: row.get(3)?,
                    file_name: row.get(4)?,
                    imported_at: row.get(5)?,
                })
            })
            .map_err(|e| LibraryError::Db(e.to_string()))?;

        let mut out = Vec::new();
        for r in rows {
            out.push(r.map_err(|e| LibraryError::Db(e.to_string()))?);
        }
        Ok(out)
    }

    /// Resolve the on-disk path for an entry (used by the audio engine).
    pub fn path_for(&self, entry: &SoundEntry) -> PathBuf {
        self.sounds_dir.join(&entry.file_name)
    }

    /// Delete an entry and its underlying file.
    pub fn delete(&self, id: &str) -> Result<(), LibraryError> {
        let conn = self.conn.lock().map_err(|e| LibraryError::Db(e.to_string()))?;
        let file_name: Option<String> = conn
            .query_row(
                "SELECT file_name FROM sounds WHERE id = ?1",
                params![id],
                |r| r.get(0),
            )
            .ok();
        if let Some(f) = file_name {
            let _ = fs::remove_file(self.sounds_dir.join(f));
        }
        conn.execute("DELETE FROM sounds WHERE id = ?1", params![id])
            .map_err(|e| LibraryError::Db(e.to_string()))?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn import_list_delete_roundtrip() {
        let tmp = std::env::temp_dir().join(format!("cacalm-test-{}", uuid::Uuid::now_v7()));
        fs::create_dir_all(&tmp).unwrap();

        let src = tmp.join("rain.mp3");
        fs::write(&src, b"not-real-audio").unwrap();

        let lib = SoundLibrary::open_at(tmp.clone()).unwrap();
        let entry = lib.import(&src, "Rain", "nature").unwrap();
        assert_eq!(entry.name, "Rain");
        assert_eq!(lib.list().unwrap().len(), 1);

        lib.delete(&entry.id).unwrap();
        assert!(lib.list().unwrap().is_empty());
        let _ = fs::remove_dir_all(&tmp);
    }

    #[test]
    fn seed_default_sounds_populates_database() {
        let tmp = std::env::temp_dir().join(format!("cacalm-seed-test-{}", uuid::Uuid::now_v7()));
        fs::create_dir_all(&tmp).unwrap();

        let lib = SoundLibrary::open_at(tmp.clone()).unwrap();
        assert_eq!(lib.list().unwrap().len(), 0);

        lib.seed_default_sounds().unwrap();
        let list = lib.list().unwrap();
        assert_eq!(list.len(), 8);

        // Verify lookup by id
        let sound = lib.get_sound_by_id("ch-rain").unwrap();
        assert!(sound.is_some());
        assert_eq!(sound.unwrap().name, "Rain");
        let _ = fs::remove_dir_all(&tmp);
    }
}
