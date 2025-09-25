use std::{env, fs};
use std::fs::File;
use std::path::{Path, PathBuf};

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
fn can_write_dir(path: &Path) -> bool {
    if !path.exists() {
        return false;
    }

    let test_file = path.join(".anLocales");
    match File::create(&test_file) {
        Ok(_) => {
            let _ = fs::remove_file(test_file);
            true
        }
        Err(_) => false,
    }
}
fn can_write_file(path: &Path) -> bool {
    match fs::OpenOptions::new().write(true).open(path) {
        Ok(_) => true,
        Err(_) => false,
    }
}

pub fn default_paths() -> (PathBuf, PathBuf, PathBuf) {
    #[cfg(unix)]
    let system_base = PathBuf::from("/usr/share/anlocales");
    #[cfg(windows)]
    let system_base = PathBuf::from("C:\\ProgramData\\anlocales");

    let use_system = is_root() || can_write_dir(&system_base);

    if use_system {
        (
            system_base.join("locales"),
            system_base.join("temp"),
            system_base.join("settings.json"),
        )
    } else {
        #[cfg(unix)]
        let home_base = env::var("HOME").unwrap_or_else(|_| "/tmp".into());
        #[cfg(windows)]
        let home_base = env::var("APPDATA").unwrap_or_else(|_| "C:\\Users\\Default\\AppData\\Roaming".into());

        let user_base = PathBuf::from(home_base).join("anlocales");
        (
            user_base.join("locales"),
            user_base.join("temp"),
            user_base.join("settings.json"),
        )
    }
}
