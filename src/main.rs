fn main() {
    println!("Hello, world!");
}

#[macro_use]
extern crate lazy_static;

use regex::Regex;
use itertools::join;
use unicode_segmentation::UnicodeSegmentation;

fn make_initials(authors: &str) -> String {
    lazy_static! {
        static ref SEP: Regex = Regex::new(r"[\s.]+").unwrap();
        static ref HYPH: Regex = Regex::new(r"\s*(?:-\s*)+").unwrap();
        static ref MON: Regex = Regex::new(r#""(?:\\.|[^"\\])*""#).unwrap();
    }

    let _first_grapheme = |s| {
        let g = UnicodeSegmentation::graphemes(s, true).collect::<Vec<&str>>();
        g[0]
    };

    let _by_space = |s| join(SEP.split(s).filter(|x| !x.is_empty()).map(|x| _first_grapheme(x).to_uppercase()), ".");
    let _by_hyph = |s| [join(HYPH.split(s).map(|x| _by_space(x)), "-"), ".".to_string()].concat();

    let _sans_monikers = MON.replace_all(authors, " ");

    join(_sans_monikers.split(",").map(_by_hyph), ",")
}

fn str_strip_numbers(s: &str) -> Vec<i64> {
    lazy_static! {
        static ref RE: Regex = Regex::new(r"\d+").unwrap();
    }
    // iterate over all matches
    RE.find_iter(s)
        .filter_map(|digits| digits.as_str().parse().ok())  // Filter out numbers out of ixx range.
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stripping_numbers() {
        assert_eq!(str_strip_numbers("ab11cdd2k.144"), vec![11, 2, 144]);
        assert!(str_strip_numbers("Ignacio Vazquez-Abrams").is_empty());
    }
    #[test]
    fn making_initials() {
        assert_eq!(make_initials(" "), ".");
        assert_eq!(make_initials("John ronald reuel Tolkien"), "J.R.R.T.");
        assert_eq!(make_initials("  e.B.Sledge "), "E.B.S.");
        assert_eq!(make_initials("Apsley Cherry-Garrard"), "A.C-G.");
        assert_eq!(make_initials("Windsor Saxe-\tCoburg - Gotha"), "W.S-C-G.");
        assert_eq!(make_initials("Elisabeth Kubler-- - Ross"), "E.K-R.");
        assert_eq!(make_initials("  Fitz-Simmons Ashton-Burke Leigh"), "F-S.A-B.L.");
        assert_eq!(make_initials("Arleigh \"31-knot\"Burke "), "A.B.");
        assert_eq!(make_initials("Harry \"Bing\" Crosby, Kris \"Tanto\" Paronto"), "H.C.,K.P.");
        assert_eq!(make_initials("a.s.,b.s."), "A.S.,B.S.");
        assert_eq!(make_initials("A. Strugatsky, B...Strugatsky."), "A.S.,B.S.");
        assert_eq!(make_initials("Иржи Кропачек, Йозеф Новотный"), "И.К.,Й.Н.");
        assert_eq!(make_initials("österreich"), "Ö.");
    }
}
