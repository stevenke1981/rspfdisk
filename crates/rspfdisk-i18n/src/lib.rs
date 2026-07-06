//! Simple i18n for rust-spfdisk.
//!
//! Loads JSON locale files at compile time via `include_str!()`.
//! Supports zh-TW (default) and en locales at runtime.
//!
//! Usage:
//! ```ignore
//! use rspfdisk_i18n::t;
//! rspfdisk_i18n::set_locale("en");
//! println!("{}", t!("main.title"));
//! ```

use std::collections::HashMap;
use std::sync::Mutex;

/// Runtime locale data loaded at compile time.
struct I18nData {
    zh_tw: HashMap<String, String>,
    en: HashMap<String, String>,
}

static DATA: once_cell::sync::Lazy<I18nData> = once_cell::sync::Lazy::new(|| I18nData {
    zh_tw: serde_json::from_str(include_str!("../locales/zh-TW.json"))
        .expect("zh-TW.json is valid JSON"),
    en: serde_json::from_str(include_str!("../locales/en.json")).expect("en.json is valid JSON"),
});

static CURRENT_LOCALE: Mutex<String> = Mutex::new(String::new());

/// Set the active locale. Supported: "zh-TW", "en".
/// Falls back to "en" for unknown values.
pub fn set_locale(locale: &str) {
    let lc = match locale {
        "zh-TW" | "zh" | "zh_CN" => "zh-TW",
        _ => "en",
    };
    *CURRENT_LOCALE.lock().unwrap() = lc.to_string();
}

/// Get the current locale string.
pub fn current_locale() -> String {
    let l = CURRENT_LOCALE.lock().unwrap();
    if l.is_empty() {
        "zh-TW".to_string()
    } else {
        l.clone()
    }
}

/// Translate a key to the current locale.
/// Falls back to the key itself if not found.
pub fn tr(key: &str) -> String {
    let locale_str = current_locale();
    let is_zh = locale_str == "zh-TW";
    let map = if is_zh { &DATA.zh_tw } else { &DATA.en };
    map.get(key).cloned().unwrap_or_else(|| {
        // Fallback: try the other locale
        let other = if is_zh { &DATA.en } else { &DATA.zh_tw };
        other.get(key).cloned().unwrap_or_else(|| key.to_string())
    })
}

/// Translate a key with positional arguments using `{0}`, `{1}`, etc.
/// Falls back to `tr()` for simple keys.
pub fn tr_fmt(key: &str, args: &[&str]) -> String {
    let mut s = tr(key);
    for (i, arg) in args.iter().enumerate() {
        s = s.replace(&format!("{{{i}}}"), arg);
    }
    s
}

#[macro_export]
macro_rules! t {
    ($key:expr $(, $arg:expr)* $(,)?) => {{
        let __s = $crate::tr($key);
        $(
            let __s = __s.replacen("{}", $arg, 1);
        )*
        __s
    }};
}

/// Detect locale from environment variable `RSPFDISK_LANG`.
/// If not set, defaults to zh-TW.
pub fn detect_locale() -> String {
    std::env::var("RSPFDISK_LANG")
        .ok()
        .and_then(|v| {
            let v = v.trim().to_lowercase();
            if v.starts_with("zh") {
                Some("zh-TW".to_string())
            } else if v.starts_with("en") {
                Some("en".to_string())
            } else {
                None
            }
        })
        .unwrap_or_else(|| "zh-TW".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn with_locale<F: FnOnce()>(locale: &str, f: F) {
        set_locale(locale);
        f();
    }

    #[test]
    fn test_zh_default() {
        with_locale("zh-TW", || {
            let s = tr("main.title");
            assert_eq!(s, "主選單");
        });
    }

    #[test]
    fn test_en_switch() {
        with_locale("en", || {
            let s = tr("main.title");
            assert_eq!(s, "Main Menu");
        });
    }

    #[test]
    fn test_fallback_key() {
        with_locale("zh-TW", || {
            let s = tr("nonexistent_key_xyz");
            assert_eq!(s, "nonexistent_key_xyz");
        });
    }

    #[test]
    fn test_macro_no_args() {
        with_locale("en", || {
            let s = t!("main.title");
            assert_eq!(s, "Main Menu");
        });
    }

    #[test]
    fn test_macro_with_arg() {
        with_locale("en", || {
            let s = t!("main.target_disk", "/dev/sda");
            assert_eq!(s, "Target Disk: /dev/sda");
        });
    }

    #[test]
    fn test_macro_multiple_args() {
        with_locale("en", || {
            let s = t!(
                "part_table.part_entry",
                "1",
                "ESP",
                "0.50",
                "Esp",
                "2048",
                "1048576"
            );
            assert_eq!(s, "[1] ESP  0.50 GiB  Esp  LBA 2048-1048576");
        });
    }

    #[test]
    fn test_detect_locale() {
        std::env::set_var("RSPFDISK_LANG", "en");
        assert_eq!(detect_locale(), "en");
        std::env::set_var("RSPFDISK_LANG", "zh-TW");
        assert_eq!(detect_locale(), "zh-TW");
        std::env::remove_var("RSPFDISK_LANG");
    }
}
