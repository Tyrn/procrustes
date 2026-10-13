use crate::icons::{BDELIM_ICON, LINK_ICON};
use crate::KNOWN_EXTENSIONS;
use regex::Regex;
use std::{cmp, ffi::OsStr, path::Path, sync::LazyLock};

/// Returns a human readable string representation of [bytes], nicely rounded.
///
pub fn human_fine(bytes: u64) -> String {
    static UNIT_LIST: [(&str, i32); 6] = [
        ("", 0),
        ("kB", 0),
        ("MB", 1),
        ("GB", 2),
        ("TB", 2),
        ("PB", 2),
    ];
    let fb = bytes as f64;
    if bytes > 1 {
        let exponent = cmp::min(fb.log(1024.0) as i32, UNIT_LIST.len() as i32 - 1);
        let quotient = fb / 1024.0_f64.powi(exponent);
        return match UNIT_LIST[exponent as usize] {
            (unit, 0) => format!("{:.0}{}", quotient, unit),
            (unit, 1) => format!("{:.1}{}", quotient, unit),
            (unit, 2) => format!("{:.2}{}", quotient, unit),
            _ => panic!(
                "{}Fatal error: human_fine(): unexpected decimals count.{}",
                BDELIM_ICON, BDELIM_ICON
            ),
        };
    }
    if bytes == 0 {
        return "0".to_string();
    }
    if bytes == 1 {
        return "1".to_string();
    }
    panic!(
        "{}Fatal error: human_fine({}).{}",
        BDELIM_ICON, bytes, BDELIM_ICON
    )
}

/// Shrinks [s] to the [limit], removing an arbitrary
/// slice from the middle.
///
pub fn str_shrink(s: &str, limit: usize) -> String {
    let s: Vec<char> = s.chars().collect();
    let limit = cmp::max(10, limit);
    if s.len() > limit {
        let (head, tail) = s.split_at(s.len() / 2);
        let (hh, _) = head.split_at(limit / 2);
        let (_, tt) = tail.split_at(tail.len() - limit / 2);
        return format!(
            "{} {} {}",
            hh.iter().collect::<String>().trim(),
            LINK_ICON,
            tt.iter().collect::<String>().trim()
        );
    }
    s.into_iter().collect()
}

/// Returns true, if [path] has an audio file extension, otherwise false.
///
pub fn is_audiofile_ext(path: &Path) -> bool {
    KNOWN_EXTENSIONS
        .iter()
        .any(|ext| has_ext_of(path.to_str().unwrap(), ext))
}

pub fn has_ext_of(path: &str, ext: &str) -> bool {
    let p = path.to_uppercase();
    let e = ext.to_uppercase().replace('.', "");
    Path::new(&p).extension() == Some(OsStr::new(&e))
}

#[allow(dead_code)]
/// Returns a vector of integer numbers, embedded in [s].
///
pub fn str_strip_numbers(s: &str) -> Vec<i64> {
    static RE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"\d+").unwrap());
    // iterate over all matches
    RE.find_iter(s)
        .filter_map(|digits| digits.as_str().parse().ok()) // Filter out numbers out of ixx range.
        .collect()
}
