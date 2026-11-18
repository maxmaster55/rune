//! Android: system font files named by the NDK's font API (API 29), the platform's own list and
//! fallback, as Skia's `SkFontMgr_AndroidNDK` asks it. `ASystemFontIterator` lists the files the
//! system's font configuration names, without opening them; `AFontMatcher` picks the font Android
//! falls back to for a run of text in a locale. No font folder is walked and no file is parsed
//! here: a file is opened only when the app loads it.
//!
//! The file-name matching is plain Rust and its tests run on the desktop; the NDK calls are
//! Android's alone.

use std::path::{Path, PathBuf};
#[cfg(target_os = "android")]
use std::{ffi::CString, sync::OnceLock};

#[cfg(target_os = "android")]
use ndk::font::{FontMatcher, SystemFontIterator};

/// The family Android's `fonts.xml` names its default sans; its fallback chain covers every
/// script the system has a font for.
#[cfg(target_os = "android")]
const SANS: &std::ffi::CStr = c"sans-serif";

/// Every system font file once, with its family key (`family_key`), in the order the system
/// lists them. A file listed for several styles or locales appears once.
#[cfg(target_os = "android")]
fn system_files() -> &'static [(String, PathBuf)] {
    static FILES: OnceLock<Vec<(String, PathBuf)>> = OnceLock::new();
    FILES.get_or_init(|| {
        let mut out: Vec<(String, PathBuf)> = Vec::new();
        for font in SystemFontIterator::new().into_iter().flatten() {
            let path = font.path();
            if out.iter().any(|(_, p)| p == path) {
                continue;
            }
            if let Some(key) = family_key(path) {
                out.push((key, path.to_path_buf()));
            }
        }
        out
    })
}

/// A font file's family as its name spells it: the stem before the first `-`, lowercase,
/// letters and digits only. Android names its files `<Family>-<Style>.<ext>`
/// (`NotoSansArabic-Bold.ttf`, `NotoColorEmoji.ttf`, `NotoSansCJK-Regular.ttc`).
pub(super) fn family_key(path: &Path) -> Option<String> {
    let stem = path.file_stem()?.to_str()?;
    let family = stem.split('-').next().unwrap_or(stem);
    let key = name_key(family);
    (!key.is_empty()).then_some(key)
}

/// A family name as `family_key` spells a file's: `Noto Sans Arabic` is `notosansarabic`.
pub(super) fn name_key(name: &str) -> String {
    name.chars().filter(char::is_ascii_alphanumeric).map(|c| c.to_ascii_lowercase()).collect()
}

/// The files whose family key is `key`, in the system's order, added to `out` once each.
pub(super) fn files_of_key(files: &[(String, PathBuf)], key: &str, out: &mut Vec<PathBuf>) {
    for (k, path) in files {
        if k == key && !out.contains(path) {
            out.push(path.clone());
        }
    }
}

/// The files of the installed families `names` (any case), in order: each family's every file,
/// so a label takes its Medium and a heading its Bold.
#[cfg(target_os = "android")]
pub(super) fn family_files(names: &[&str]) -> Vec<PathBuf> {
    let files = system_files();
    let mut out = Vec::new();
    for name in names {
        files_of_key(files, &name_key(name), &mut out);
    }
    out
}

/// The files of the fonts Android falls back to for `chars` in `locale`, each family whole.
/// The matcher answers for a run at a time, so a string of one script costs one call. A
/// character no installed font covers gets the default sans; loading it costs a mapping once,
/// and `TextSystem::load_fallbacks` asks for each character only once.
#[cfg(target_os = "android")]
pub(super) fn fallback_family_files(chars: &[char], locale: Option<&str>) -> Vec<PathBuf> {
    let text: Vec<u16> = chars.iter().flat_map(|ch| ch.encode_utf16(&mut [0; 2]).to_vec()).collect();
    if text.is_empty() {
        return Vec::new();
    }
    let mut matcher = FontMatcher::new();
    if let Some(tags) = locale.and_then(|l| CString::new(l).ok()) {
        matcher.set_locales(&tags);
    }
    let files = system_files();
    let mut out: Vec<PathBuf> = Vec::new();
    let mut at = 0;
    while at < text.len() {
        let mut run = 0u32;
        let font = matcher.match_font(SANS, &text[at..], Some(&mut run));
        let path = font.path();
        match family_key(path) {
            Some(key) if files.iter().any(|(k, _)| *k == key) => files_of_key(files, &key, &mut out),
            _ if !out.iter().any(|p| p == path) => out.push(path.to_path_buf()),
            _ => {}
        }
        // A run is at least one UTF-16 unit, so the loop always advances.
        at += (run as usize).max(1);
    }
    out
}

#[cfg(test)]
#[path = "android_tests.rs"]
mod tests;
