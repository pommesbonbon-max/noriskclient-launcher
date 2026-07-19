use directories::ProjectDirs;
use once_cell::sync::Lazy;
use reqwest::Client;
use std::path::PathBuf;
use std::sync::RwLock;

/// Either the OS-standard application directories, or a portable directory
/// located next to the launcher executable.
pub enum LauncherDirs {
    Standard(ProjectDirs),
    Portable(PathBuf),
}

/// Returns the portable base directory if a `portable.txt` marker file
/// exists next to the launcher executable (same convention as e.g. Prism
/// Launcher's portable builds).
fn portable_base_dir() -> Option<PathBuf> {
    let exe = std::env::current_exe().ok()?;
    let exe_dir = exe.parent()?.to_path_buf();
    if exe_dir.join("portable.txt").exists() {
        Some(exe_dir.join("NoRiskClientData"))
    } else {
        None
    }
}

pub static LAUNCHER_DIRECTORY: Lazy<LauncherDirs> = Lazy::new(|| {
    if let Some(base) = portable_base_dir() {
        return LauncherDirs::Portable(base);
    }

    match ProjectDirs::from("gg", "norisk", "NoRiskClientV3") {
        Some(proj_dirs) => LauncherDirs::Standard(proj_dirs),
        None => panic!("Failed to get application directory"),
    }
});

impl LauncherDirs {
    pub fn data_dir(&self) -> PathBuf {
        match self {
            LauncherDirs::Standard(p) => p.data_dir().to_path_buf(),
            LauncherDirs::Portable(base) => base.clone(),
        }
    }

    pub fn cache_dir(&self) -> PathBuf {
        match self {
            LauncherDirs::Standard(p) => p.cache_dir().to_path_buf(),
            LauncherDirs::Portable(base) => base.join("cache"),
        }
    }
}

pub static CUSTOM_GAME_DIR_CACHE: Lazy<RwLock<Option<Option<PathBuf>>>> = 
    Lazy::new(|| RwLock::new(None));

static APP_USER_AGENT: &str = concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION"),);

/// HTTP Client with launcher agent
pub static HTTP_CLIENT: Lazy<Client> = Lazy::new(|| {
    let client = reqwest::ClientBuilder::new()
        .user_agent(APP_USER_AGENT)
        .build()
        .unwrap_or_else(|_| Client::new());
    client
});

// Extension trait for LauncherDirs to add meta_dir functionality
pub trait ProjectDirsExt {
    fn meta_dir(&self) -> PathBuf;
    fn root_dir(&self) -> PathBuf;
}

impl ProjectDirsExt for LauncherDirs {
    fn meta_dir(&self) -> PathBuf {
        // Check cache first
        if let Ok(guard) = CUSTOM_GAME_DIR_CACHE.read() {
            if let Some(cached_value) = guard.as_ref() {
                if let Some(custom_dir) = cached_value {
                    return custom_dir.clone();
                }
            }
        }
        
        // Fallback to standard logic
        standard_meta_dir()
    }

    fn root_dir(&self) -> PathBuf {
        match self {
            // Portable installs keep everything in one folder next to the exe
            LauncherDirs::Portable(base) => base.clone(),
            LauncherDirs::Standard(p) => {
                if cfg!(target_os = "windows") {
                    // Windows: Alte Logik (wie sie war)
                    p.data_dir().parent().unwrap().to_path_buf()
                } else {
                    // macOS (und andere): Setze root_dir auf data_dir
                    p.data_dir().to_path_buf()
                }
            }
        }
    }
}

/// Returns the standard meta directory (ignores custom directory setting)
/// Used for Java and other system components that need to stay in standard location due to macos...x
pub fn standard_meta_dir() -> PathBuf {
    LAUNCHER_DIRECTORY.root_dir().join("meta")
}

/// Update the cached custom game directory
pub fn update_custom_game_dir(path: Option<PathBuf>) {
    if let Ok(mut guard) = CUSTOM_GAME_DIR_CACHE.write() {
        *guard = Some(path);
    }
}
