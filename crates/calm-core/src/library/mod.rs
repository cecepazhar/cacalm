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
    pub fn open() -> Result<Self, LibraryError> {        let data_dir = app_data_dir().map_err(|e| LibraryError::Io(e.to_string()))?;
        let sounds_dir = data_dir.join("sounds");
        fs::create_dir_all(&sounds_dir).map_err(|e| LibraryError::Io(e.to_string()))?;

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

        Ok(Self { conn: Mutex::new(conn), sounds_dir })
    }

    /// Copy `source` into the library and record it. Returns the new entry.
    pub fn import(&self, source: &Path, name: &str, category: &str) -> Result<SoundEntry, LibraryError> {
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
            .query_row("SELECT file_name FROM sounds WHERE id = ?1", params![id], |r| r.get(0))
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
        // Isolate the test from the user's real library by pointing HOME at a temp dir.
        let tmp = std::env::temp_dir().join(format!("cacalm-test-{}", uuid::Uuid::now_v7()));
        fs::create_dir_all(&tmp).unwrap();
        // SAFETY: single-threaded test, no other reader of HOME.
        unsafe { std::env::set_var("HOME", &tmp) };

        let src = tmp.join("rain.mp3");
        fs::write(&src, b"not-real-audio").unwrap();

        let lib = SoundLibrary::open().unwrap();
        let entry = lib.import(&src, "Rain", "nature").unwrap();
        assert_eq!(entry.name, "Rain");
        assert_eq!(lib.list().unwrap().len(), 1);

        lib.delete(&entry.id).unwrap();
        assert!(lib.list().unwrap().is_empty());
    }
}
