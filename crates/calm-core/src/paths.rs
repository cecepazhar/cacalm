use std::path::PathBuf;

/// Application data directory.
/// On Linux: ~/.local/share/cacalm
/// On macOS: ~/Library/Application Support/com.cafamily.calm
/// On Windows: %APPDATA%\cacalm
pub fn app_data_dir() -> Result<PathBuf, std::io::Error> {
    let mut path = directories::ProjectDirs::from(
        "com",      // domain
        "Fathforce", // organization
        "CACalm",    // application name
    )
    .ok_or_else(|| std::io::Error::new(std::io::ErrorKind::Other, "failed to determine app data dir"))?;

    // Ensure directory exists
    std::fs::create_dir_all(path.path())?;
    Ok(path.data_dir().to_path_buf())
}
