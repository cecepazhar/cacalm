use std::path::PathBuf;

/// Application data directory.
/// On Linux: ~/.local/share/cacalm
/// On macOS: ~/Library/Application Support/com.fathforce.cacalm
/// On Windows: %APPDATA%\cacalm
pub fn app_data_dir() -> Result<PathBuf, std::io::Error> {
    let dirs = directories::ProjectDirs::from("com", "Fathforce", "CACalm")
        .ok_or_else(|| {
            std::io::Error::new(
                std::io::ErrorKind::Other,
                "failed to determine app data dir",
            )
        })?;

    let data_dir = dirs.data_dir();
    std::fs::create_dir_all(data_dir)?;
    Ok(data_dir.to_path_buf())
}