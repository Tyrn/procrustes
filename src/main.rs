#[macro_use]
extern crate lazy_static;

use clap::{Arg, ArgMatches, App, SubCommand};
use regex::Regex;
use itertools::join;
use unicode_segmentation::UnicodeSegmentation;
use std::path::Path;
use std::ffi::OsStr;

const APP_DESCRIPTION: &str = r#"
pcr "Procrustes" SmArT is a CLI utility for copying subtrees containing supported audio
files in sequence, naturally sorted.
The end result is a "flattened" copy of the source subtree. "Flattened" means
that only a namesake of the root source directory is created, where all the files get
copied to, names prefixed with a serial number. Tag "Track Number"
is set, tags "Title", "Artist", and "Album" can be replaced optionally.
The writing process is strictly sequential: either starting with the number one file,
or in the reversed order. This can be important for some mobile devices.
"#;

lazy_static! {
    static ref ARGS: ArgMatches<'static> = retrieve_args();
}

fn retrieve_args() -> ArgMatches<'static> {
    App::new(APP_DESCRIPTION)
        .version("1.0")
        .author("")
        .about("")
        .arg(Arg::with_name("config")
           .short("c")
           .long("config")
           .value_name("FILE")
           .help("Sets a custom config file")
           .takes_value(true))
        .arg(Arg::with_name("INPUT")
           .help("Sets the input file to use")
           .required(true)
           .index(1))
        .arg(Arg::with_name("v")
           .short("v")
           .multiple(true)
           .help("Sets the level of verbosity"))
        .subcommand(SubCommand::with_name("test")
                  .about("controls testing features")
                  .version("1.3")
                  .author("Someone E. <someone_else@other.com>")
                  .arg(Arg::with_name("debug")
                      .short("d")
                      .help("print debug information verbosely")))
        .get_matches()
}

fn main() {
    println!("[[[{}]]]", ARGS.value_of("INPUT").unwrap());
}

#[allow(dead_code)]
fn has_ext_of(path: &str, ext: &str) -> bool {
    let p = path.to_uppercase();
    let e = ext.to_uppercase().replace(".", "");
    Path::new(&p).extension() == Some(OsStr::new(&e))
}

#[allow(dead_code)]
fn str_strip_numbers(s: &str) -> Vec<i64> {
    lazy_static! {
        static ref RE: Regex = Regex::new(r"\d+").unwrap();
    }
    // iterate over all matches
    RE.find_iter(s)
        .filter_map(|digits| digits.as_str().parse().ok())  // Filter out numbers out of ixx range.
        .collect()
}

#[allow(dead_code)]
fn make_initials(authors: &str) -> String {
    lazy_static! {
        static ref SEP: Regex = Regex::new(r"[\s.]+").unwrap();
        static ref HYPH: Regex = Regex::new(r"\s*(?:-\s*)+").unwrap();
        static ref MON: Regex = Regex::new(r#""(?:\\.|[^"\\])*""#).unwrap();
    }

    let first_grapheme = |s| {
        let g = UnicodeSegmentation::graphemes(s, true).collect::<Vec<&str>>();
        g[0]
    };

    let by_space = |s| join(SEP.split(s).filter(|x| !x.is_empty()).map(|x| first_grapheme(x).to_uppercase()), ".");
    let by_hyph = |s| [join(HYPH.split(s).map(|x| by_space(x)), "-"), ".".to_string()].concat();

    let sans_monikers = MON.replace_all(authors, " ");

    join(sans_monikers.split(",").map(by_hyph), ",")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn checking_ext_of() {
        assert_eq!(has_ext_of("/alfa/bra.vo/charlie.ogg", "OGG"), true);
        assert_eq!(has_ext_of("/alfa/bra.vo/charlie.ogg", ".ogg"), true);
        assert_eq!(has_ext_of("/alfa/bra.vo/charlie.ogg", "mp3"), false);
    }
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
