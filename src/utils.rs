use std::env;
use std::fs::File;
use std::path::PathBuf;

// utils

// windows
#[cfg(windows)]
use is_admin;
#[cfg(windows)]
fn is_root() -> bool {
    is_admin::is_admin()
}

// unix
#[cfg(unix)]
use nix::unistd::Uid;
use crate::Settings;

#[cfg(unix)]
fn is_root() -> bool {
    Uid::effective().is_root()
}

// config or paths
pub fn ensure_that_config_exists(settings_file_path: PathBuf) {
    if !settings_file_path.exists() {
        let default_settings = Settings {
            default_locale: "en_US".into(),
            fallback_locale: "en_US".into(),
        };
        let file = File::create(&settings_file_path).unwrap();
        serde_json::to_writer(file, &default_settings).unwrap();
    }
}

pub(crate) fn default_paths() -> (PathBuf, PathBuf, PathBuf) {
    let is_admin = is_root();

    if is_admin {
        #[cfg(unix)]
        return (
            PathBuf::from("/usr/share/anlocales/locales"),
            PathBuf::from("/usr/share/anlocales/temp"),
            PathBuf::from("/usr/share/anlocales/settings.json"),
        );

        #[cfg(windows)]
        return (
            PathBuf::from("C:\\ProgramData\\anlocales\\locales"),
            PathBuf::from("C:\\ProgramData\\anlocales\\temp"),
            PathBuf::from("C:\\ProgramData\\anlocales\\settings.json"),
        )
    } else {
        #[cfg(unix)]
        let base_dir = env::var("HOME").unwrap_or_else(|_| "/tmp".into());

        #[cfg(windows)]
        let base_dir = env::var("APPDATA").unwrap_or_else(|_| "C:\\Users\\Default\\AppData\\Roaming".into());

        (
            PathBuf::from(format!("{}/anlocales/locales", base_dir)),
            PathBuf::from(format!("{}/anlocales/temp", base_dir)),
            PathBuf::from(format!("{}/anlocales/settings.json", base_dir)),
        )
    }
}
