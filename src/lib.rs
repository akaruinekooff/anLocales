mod data_format;
mod utils;

use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::ffi::{CStr, CString};
use std::fs::{self, File};
use std::os::raw::c_char;
use std::path::{Path, PathBuf};
use std::sync::Arc;

#[cfg(windows)]
use is_admin;

#[derive(Deserialize, Debug, Serialize)]
pub struct Settings {
    pub default_locale: String,
    pub fallback_locale: String,
}


#[derive(Clone)]
pub struct Locale {
    pub data: data_format::DataFormat,
    pub strings: HashMap<String, Vec<String>>,
    pub plural_fn: Arc<dyn Fn(u32) -> usize + Send + Sync>,
    pub name: String,
}

impl Locale {
    pub fn load(path: &Path, name: &str) -> Self {
        let data_file = File::open(path.join("data_format.json")).expect("data_format.json not found");
        let data: data_format::DataFormat = serde_json::from_reader(data_file).expect("Failed to parse data_format.json");

        let toml_str = fs::read_to_string(path.join("locale.toml")).expect("locale.toml not found");
        let strings: HashMap<String, Vec<String>> = toml::from_str(&toml_str).expect("Failed to parse locale.toml");

        let plural_rule = data.PLURAL_RULES.clone();
        let plural_fn = Arc::new(move |n: u32| {
            let expr = plural_rule.replace("n", &n.to_string());
            meval::eval_str(&expr).unwrap_or(0.0) as usize
        });

        Self { data, strings, plural_fn, name: name.to_string() }
    }

    pub fn t<'a>(&'a self, key: &'a str) -> &'a str {
        self.strings.get(key).and_then(|v| v.get(0)).map(|s| s.as_str()).unwrap_or(key)
    }

    pub fn plural_word<'a>(&'a self, key: &'a str, n: u32) -> &'a str {
        if let Some(forms) = self.strings.get(key) {
            let idx = (self.plural_fn)(n);
            &forms[std::cmp::min(idx, forms.len() - 1)]
        } else {
            key
        }
    }

    pub fn format_date(&self, date: Option<NaiveDate>) -> String {
        date.unwrap().format(&self.data.LC_TIME.date_fmt).to_string()
    }

    pub fn format_money(&self, amount: f64) -> String {
        let fmt = &self.data.LC_MONETARY;
        format!("{}{:.*}", fmt.currency_symbol, fmt.frac_digits as usize, amount)
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
    pub fn new() -> Self {
        // hook for panic
        std::panic::set_hook(Box::new(|info| {
            eprintln!("panic happened: {}", info);
        }));

        // directory
        let (locales_path, temp_path, settings_file_path) = utils::default_paths();

        // init
        fs::create_dir_all(&locales_path).expect("failed to create locales dir");
        fs::create_dir_all(&temp_path).expect("failed to create temp dir");
        utils::ensure_that_config_exists(settings_file_path.clone());

        // opening and parsing settings.json
        let settings_file = File::open(&settings_file_path).expect("settings.json not found");
        let settings: Settings = serde_json::from_reader(settings_file).expect("Failed to parse settings.json");

        Self { locales_path, temp_path, settings, cache: HashMap::new() }
    }

    pub fn new_with_paths(locales_path : PathBuf, temp_path : PathBuf, settings_file_path : PathBuf) -> Self {
        // hook for panic
        std::panic::set_hook(Box::new(|info| {
            eprintln!("panic happened: {}", info);
        }));

        // init
        fs::create_dir_all(&locales_path).expect("failed to create locales dir");
        fs::create_dir_all(&temp_path).expect("failed to create temp dir");
        utils::ensure_that_config_exists(settings_file_path.clone());

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
pub extern "C" fn anlocales_new_with_paths(
    locales_path: *const c_char,
    temp_path: *const c_char,
    settings_file_path: *const c_char,
) -> *mut AnLocales {
    if locales_path.is_null() || temp_path.is_null() || settings_file_path.is_null() {
        return std::ptr::null_mut();
    }

    let locales_path = unsafe { CStr::from_ptr(locales_path).to_string_lossy().into_owned() };
    let temp_path = unsafe { CStr::from_ptr(temp_path).to_string_lossy().into_owned() };
    let settings_file_path = unsafe { CStr::from_ptr(settings_file_path).to_string_lossy().into_owned() };

    let al = AnLocales::new_with_paths(
        PathBuf::from(locales_path),
        PathBuf::from(temp_path),
        PathBuf::from(settings_file_path),
    );

    Box::into_raw(Box::new(al))
}

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
pub unsafe extern "C" fn anlocales_default_locale(ptr: *mut AnLocales) -> *mut Locale {
    unsafe {
        if ptr.is_null() { return std::ptr::null_mut(); }
        let al = &mut *ptr;
        let locale = al.default_locale();
        Box::into_raw(Box::new(locale.clone()))
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn anlocales_fallback_locale(ptr: *mut AnLocales) -> *mut Locale {
    unsafe {
        if ptr.is_null() { return std::ptr::null_mut(); }
        let al = &mut *ptr;
        let locale = al.fallback_locale();
        Box::into_raw(Box::new(locale.clone()))
    }
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
    let s1 = unsafe { CStr::from_ptr(a).to_str().unwrap_or("") };
    let s2 = unsafe { CStr::from_ptr(b).to_str().unwrap_or("") };
    let locale = unsafe { &*ptr };
    locale.compare(s1, s2)
}

#[unsafe(no_mangle)]
pub extern "C" fn locale_plural_word(ptr: *mut Locale, key: *const c_char, n: u32) -> *const c_char {
    if ptr.is_null() || key.is_null() { return std::ptr::null(); }
    let locale = unsafe { &*ptr };
    let key_str = unsafe { CStr::from_ptr(key) }.to_str().unwrap();
    let word = locale.plural_word(key_str, n);
    CString::new(word).unwrap().into_raw()
}

#[unsafe(no_mangle)]
pub extern "C" fn locale_free_str(s: *mut c_char) {
    if s.is_null() { return; }
    unsafe { let _ = CString::from_raw(s); }
}
