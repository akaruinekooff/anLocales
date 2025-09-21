mod data_format;

use std::collections::HashMap;
use std::env;
use std::ffi::{CString, CStr};
use std::os::raw::c_char;
use std::fs::{self, File};
use std::path::{Path, PathBuf};
use serde::{Deserialize, Serialize};
use chrono::NaiveDate;

#[cfg(unix)]
use nix::unistd::Uid;

#[cfg(windows)]
use whoami;

#[derive(Deserialize, Debug, Serialize)]
pub struct Settings {
    pub default_locale: String,
    pub fallback_locale: String,
}

#[derive(Clone)]
pub struct Locale {
    pub data: data_format::DataFormat,
    pub strings: HashMap<String, String>,
    pub name: String,
}

impl Locale {
    fn load(path: &Path, name: &str) -> Self {
        let data_file = File::open(path.join("data_format.json")).expect("data_format.json not found");
        let data: data_format::DataFormat = serde_json::from_reader(data_file).expect("Failed to parse data_format.json");

        let toml_str = fs::read_to_string(path.join("locale.toml")).expect("locale.toml not found");
        let strings: HashMap<String, String> = toml::from_str(&toml_str).expect("Failed to parse locale.toml");

        Self { data, strings, name: name.to_string() }
    }

    pub fn t<'a>(&'a self, key: &'a str) -> &'a str {
        self.strings.get(key).map(|s| s.as_str()).unwrap_or(key)
    }

    pub fn format_date(&self, date: Option<NaiveDate>) -> String {
        date.unwrap().format(&self.data.LC_TIME.date_fmt).to_string()
    }

    pub fn format_money(&self, amount: f64) -> String {
        let fmt = &self.data.LC_MONETARY;
        format!("{}{:.*}", fmt.currency_symbol, fmt.frac_digits as usize, amount)
    }

    pub fn plural(&self, n: u32) -> bool {
        self.data.PLURAL_RULES != "n != 1" || n != 1
    }

    pub fn compare(&self, a: &str, b: &str) -> i32 {
        match self.data.LC_COLLATE.sort_order.as_str() {
            "unicode" => a.cmp(b) as i32,
            _ => a.cmp(b) as i32,
        }
    }
}

pub struct AnLocales {
    pub locales_path: PathBuf,
    pub temp_path: PathBuf,
    pub settings: Settings,
    pub cache: HashMap<String, Locale>,
}

impl AnLocales {
    fn default_paths() -> (PathBuf, PathBuf, PathBuf) {
        let is_admin = {
            #[cfg(unix)]
            { Uid::effective().is_root() }

            #[cfg(windows)]
            { whoami::privilege_level() == whoami::PrivilegeLevel::Admin }
        };

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

    pub fn new() -> Self {
        // hook for panic
        std::panic::set_hook(Box::new(|info| {
            eprintln!("panic happened: {}", info);
        }));

        // directory
        let (locales_path, temp_path, settings_file_path) = Self::default_paths();

        // init
        fs::create_dir_all(&locales_path).expect("failed to create locales dir");
        fs::create_dir_all(&temp_path).expect("failed to create temp dir");
        if !settings_file_path.exists() {
            let default_settings = Settings {
                default_locale: "en_US".into(),
                fallback_locale: "en_US".into(),
            };
            let file = File::create(&settings_file_path).unwrap();
            serde_json::to_writer(file, &default_settings).unwrap();
        }

        // opening and parsing settings.json
        let settings_file = File::open(&settings_file_path).expect("settings.json not found");
        let settings: Settings = serde_json::from_reader(settings_file).expect("Failed to parse settings.json");

        Self { locales_path, temp_path, settings, cache: HashMap::new() }
    }

    pub fn load_locale(&mut self, name: &str) -> &Locale {
        if !self.cache.contains_key(name) {
            let locale = Locale::load(&self.locales_path.join(name), name);
            self.cache.insert(name.to_string(), locale);
        }
        self.cache.get(name).unwrap()
    }

    pub fn default_locale(&mut self) -> &Locale {
        let name = self.settings.default_locale.clone();
        self.load_locale(&name)
    }

    pub fn fallback_locale(&mut self) -> &Locale {
        let name = self.settings.fallback_locale.clone();
        self.load_locale(&*name)
    }
}

// ================= C API =================

#[unsafe(no_mangle)]
pub extern "C" fn anlocales_new() -> *mut AnLocales {
    Box::into_raw(Box::new(AnLocales::new()))
}

#[unsafe(no_mangle)]
pub extern "C" fn anlocales_free(ptr: *mut AnLocales) {
    if ptr.is_null() { return; }
    unsafe { let _ = Box::from_raw(ptr); }
}

#[unsafe(no_mangle)]
pub extern "C" fn locale_load(ptr: *mut AnLocales, name: *const c_char) -> *mut Locale {
    if ptr.is_null() || name.is_null() { return std::ptr::null_mut(); }
    let cstr = unsafe { CStr::from_ptr(name) };
    let name_str = cstr.to_str().unwrap();
    let al = unsafe { &mut *ptr };
    let locale = al.load_locale(name_str);
    Box::into_raw(Box::new(locale.clone()))
}

#[unsafe(no_mangle)]
pub extern "C" fn locale_free(ptr: *mut Locale) {
    if ptr.is_null() { return; }
    unsafe { let _ = Box::from_raw(ptr); }
}

#[unsafe(no_mangle)]
pub extern "C" fn locale_t(ptr: *mut Locale, key: *const c_char) -> *const c_char {
    if ptr.is_null() || key.is_null() { return std::ptr::null(); }
    let cstr = unsafe { CStr::from_ptr(key) };
    let key_str = cstr.to_str().unwrap();
    let locale = unsafe { &*ptr };
    let val = locale.t(key_str);
    CString::new(val).unwrap().into_raw()
}

#[unsafe(no_mangle)]
pub extern "C" fn locale_format_date(ptr: *mut Locale, year: i32, month: u32, day: u32) -> *const c_char {
    if ptr.is_null() { return std::ptr::null(); }
    let locale = unsafe { &*ptr };
    let date = NaiveDate::from_ymd_opt(year, month, day);
    let s = locale.format_date(date);
    CString::new(s).unwrap().into_raw()
}

#[unsafe(no_mangle)]
pub extern "C" fn locale_format_money(ptr: *mut Locale, amount: f64) -> *const c_char {
    if ptr.is_null() { return std::ptr::null(); }
    let locale = unsafe { &*ptr };
    let s = locale.format_money(amount);
    CString::new(s).unwrap().into_raw()
}

#[unsafe(no_mangle)]
pub extern "C" fn locale_compare(ptr: *mut Locale, a: *const c_char, b: *const c_char) -> i32 {
    if ptr.is_null() || a.is_null() || b.is_null() { return 0; }
    let s1 = unsafe { CStr::from_ptr(a).to_str().unwrap() };
    let s2 = unsafe { CStr::from_ptr(b).to_str().unwrap() };
    let locale = unsafe { &*ptr };
    locale.compare(s1, s2)
}

#[unsafe(no_mangle)]
pub extern "C" fn locale_plural(ptr: *mut Locale, n: u32) -> bool {
    if ptr.is_null() { return false; }
    let locale = unsafe { &*ptr };
    locale.plural(n)
}

#[unsafe(no_mangle)]
pub extern "C" fn locale_free_str(s: *mut c_char) {
    if s.is_null() { return; }
    unsafe { let _ = CString::from_raw(s); }
}
