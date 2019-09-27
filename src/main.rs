#[macro_use]
extern crate lazy_static;

use alphanumeric_sort::sort_path_slice;
use clap::{App, Arg, ArgMatches};
use itertools::join;
use path_absolutize::*;
use regex::Regex;
use std::{
    ffi::OsStr,
    fs, io,
    path::{Path, PathBuf},
    process::exit,
};
use unicode_segmentation::UnicodeSegmentation;

const APP_DESCRIPTION: &str =
    "A CLI utility for copying subtrees containing supported \
     audio files in sequence, naturally sorted. \
     The end result is a \"flattened\" copy of the source subtree. \"Flattened\" means \
     that only a namesake of the root source directory is created, where all the files get \
     copied to, names prefixed with a serial number. Tag \"Track Number\" \
     is set, tags \"Title\", \"Artist\", and \"Album\" can be replaced optionally. \
     The writing process is strictly sequential: either starting with the number one file, \
     or in the reversed order. This can be important for some mobile devices.";

lazy_static! {
    static ref ARGS: ArgMatches<'static> = retrieve_args();
//    static ref IS_TREE: bool = is_tree_dst();
//    static ref IS_ALBUM: bool = is_album_tag();
//    static ref ALBUM: &'static str = album_tag();
    static ref SRC: PathBuf = pval("src");
    static ref DST: PathBuf = executive_dst();
}

// Creates the destination directory according to options.
// TODO: possibly call check_args() from here.
fn executive_dst() -> PathBuf {
    let prefix = if flag("b") {
        format!("{:02}-", ival("b"))
    } else {
        "".to_string()
    };
    let base_dst = format!(
        "{}{}",
        prefix,
        if flag("u") {
            format!("{}{}", artist(false), sval("u"))
        } else {
            pval("src")
                .file_name()
                .unwrap()
                .to_str()
                .unwrap()
                .to_string()
        }
    );
    if flag("p") {
        pval("dst")
    } else {
        [pval("dst"), PathBuf::from(base_dst)].iter().collect()
    }
}

fn artist(forw_dash: bool) -> String {
    if flag("a") {
        if forw_dash {
            format!("-{}", sval("a"))
        } else {
            format!("{}-", sval("a"))
        }
    } else {
        "".to_string()
    }
}

fn flag(name: &str) -> bool {
    if ARGS.occurrences_of(name) > 0 {
        true
    } else {
        false
    }
}

fn sval(name: &str) -> &str {
    ARGS.value_of(name).unwrap_or("NULL_STR")
}

fn ival(name: &str) -> i64 {
    ARGS.value_of(name)
        .unwrap_or("NULL_INT")
        .parse()
        .expect("Option value must be a valid number!")
}

fn pval(name: &str) -> PathBuf {
    Path::new(sval(name)).absolutize().unwrap()
}

fn is_tree_dst() -> bool {
    if flag("t") && flag("r") {
        false
    } else {
        flag("t")
    }
}

fn is_album_tag() -> bool {
    if flag("u") && !flag("g") {
        true
    } else {
        flag("g")
    }
}

fn album_tag() -> &'static str {
    if flag("u") && !flag("g") {
        sval("u")
    } else {
        sval("g")
    }
}

fn retrieve_args() -> ArgMatches<'static> {
    App::new("\"Procrustes\" SmArT")
        .version("1.0.2")
        .author("")
        .about(APP_DESCRIPTION)
        .arg(
            Arg::with_name("v")
                .short("v")
                .long("verbose")
                .help("verbose output"),
        )
        .arg(
            Arg::with_name("d")
                .short("d")
                .long("drop-tracknumber")
                .help("do not set track numbers"),
        )
        .arg(
            Arg::with_name("s")
                .short("s")
                .long("strip-decorations")
                .help("strip file and directory name decorations"),
        )
        .arg(
            Arg::with_name("f")
                .short("f")
                .long("file-title")
                .help("use file name for title tag"),
        )
        .arg(
            Arg::with_name("F")
                .short("F")
                .long("file-title-num")
                .help("use numbered file name for title tag"),
        )
        .arg(
            Arg::with_name("x")
                .short("x")
                .long("sort-lex")
                .help("sort files lexicographically"),
        )
        .arg(
            Arg::with_name("t")
                .short("t")
                .long("tree-dst")
                .help("retain the tree structure of the source album at destination"),
        )
        .arg(
            Arg::with_name("p")
                .short("p")
                .long("drop-dst")
                .help("do not create destination directory"),
        )
        .arg(
            Arg::with_name("r")
                .short("r")
                .long("reverse")
                .help("copy files in reverse order (number one file is the last to be copied)"),
        )
        .arg(
            Arg::with_name("i")
                .short("i")
                .long("prepend-subdir-name")
                .help("prepend current subdirectory name to a file name"),
        )
        .arg(
            Arg::with_name("e")
                .short("e")
                .long("file-type")
                .value_name("EXT")
                .help("accept only audio files of the specified type")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("u")
                .short("u")
                .long("unified-name")
                .value_name("UNIFIED_NAME")
                .help("UNIFIED_NAME for everything unspecified")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("b")
                .short("b")
                .long("album-num")
                .value_name("ALBUM_NUM")
                .help("0..99; prepend ALBUM_NUM to the destination root directory name")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("a")
                .short("a")
                .long("artist-tag")
                .value_name("ARTIST_TAG")
                .help("artist tag name")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("g")
                .short("g")
                .long("album-tag")
                .value_name("ALBUM_TAG")
                .help("album tag name")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("src")
                .help("source directory")
                .required(true)
                .index(1),
        )
        .arg(
            Arg::with_name("dst")
                .help("general destination directory")
                .required(true)
                .index(2),
        )
        .get_matches()
}

fn check_args() {
    let (src, dst) = (pval("src"), pval("dst"));

    if !src.exists() {
        println!("Source directory \"{}\" is not there.", src.display());
        exit(0);
    }
    if !dst.exists() {
        println!("Destination path \"{}\" is not there.", dst.display());
        exit(0);
    }
    if flag("t") && flag("r") {
        println!("  *** -t option ignored (conflicts with -r) ***");
    }
}

fn fs_entries(dir: &Path, folders: bool) -> Result<Vec<PathBuf>, io::Error> {
    Ok(fs::read_dir(dir)?
        .into_iter()
        .filter(|r| r.is_ok())
        .map(|r| r.unwrap().path())
        .filter(|r| if folders { r.is_dir() } else { !r.is_dir() })
        .collect())
}

#[allow(dead_code)]
fn offspring(dir: &Path) -> Result<Vec<PathBuf>, io::Error> {
    fs::read_dir(dir)?
        .into_iter()
        .map(|x| x.map(|entry| entry.path()))
        .collect()
}

fn groom(dir: &Path) -> (Vec<PathBuf>, Vec<PathBuf>) {
    let mut dirs = fs_entries(dir, true).unwrap();
    let mut files = fs_entries(dir, false).unwrap();
    if flag("x") {
        dirs.sort_unstable();
        files.sort_unstable();
    } else {
        sort_path_slice(&mut dirs);
        sort_path_slice(&mut files);
    }
    if flag("r") {
        dirs.reverse();
        files.reverse();
    }
    (dirs, files)
}

fn traverse_flat_dst(
    src_dir: &PathBuf,
    dst_step: Vec<PathBuf>,
) -> Box<dyn Iterator<Item = (PathBuf, PathBuf)>> {
    let (dirs, files) = groom(src_dir);

    let traverse = move |d: PathBuf| {
        let mut step = dst_step.clone();
        step.push(PathBuf::from(d.file_name().unwrap()));
        traverse_flat_dst(&d, step)
    };
    let handle = |f: PathBuf| {
        let dst_path: PathBuf = [&DST.as_os_str(), f.file_name().unwrap()].iter().collect();
        (f, dst_path)
    };
    if flag("r") {
        Box::new(
            files
                .into_iter()
                .map(handle)
                .chain(dirs.into_iter().flat_map(traverse)),
        )
    } else {
        Box::new(
            dirs.into_iter()
                .flat_map(traverse)
                .chain(files.into_iter().map(handle)),
        )
    }
}

fn copy_album() {
    let (dirs, files) = groom(&SRC.as_path());

    println!("dirs: {:?}", dirs);
    println!("files: {:?}", files);
    println!("folders: {:?}", offspring(&SRC.as_path()).unwrap());
    println!("||||||||||||||||||||||||");
    for (_i, (src, dst)) in traverse_flat_dst(&SRC, [].to_vec()).enumerate() {
        println!("iter(src, dst): ({:?}, {:?})", src, dst);
    }
}

fn main() {
    check_args();
    println!("VERBOSE: [{}]", flag("v"));
    println!("TREE_DST: [{}]", is_tree_dst());
    println!("EXT: [{}, {}]", flag("e"), sval("e"));
    println!("ALBUM_NUM: [{}, {}]", flag("b"), ival("b"));
    println!("ALBUM_TAG: [{}, {}]", is_album_tag(), album_tag());
    println!("SRC: [{}, {}]", flag("src"), SRC.display());
    println!("DST: [{}, {}]", flag("dst"), DST.display());
    copy_album();
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
        .filter_map(|digits| digits.as_str().parse().ok()) // Filter out numbers out of ixx range.
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

    let by_space = |s| {
        join(
            SEP.split(s)
                .filter(|x| !x.is_empty())
                .map(|x| first_grapheme(x).to_uppercase()),
            ".",
        )
    };
    let by_hyph = |s| {
        [
            join(HYPH.split(s).map(|x| by_space(x)), "-"),
            ".".to_string(),
        ]
        .concat()
    };

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
        assert_eq!(
            make_initials("  Fitz-Simmons Ashton-Burke Leigh"),
            "F-S.A-B.L."
        );
        assert_eq!(make_initials("Arleigh \"31-knot\"Burke "), "A.B.");
        assert_eq!(
            make_initials("Harry \"Bing\" Crosby, Kris \"Tanto\" Paronto"),
            "H.C.,K.P."
        );
        assert_eq!(make_initials("a.s.,b.s."), "A.S.,B.S.");
        assert_eq!(make_initials("A. Strugatsky, B...Strugatsky."), "A.S.,B.S.");
        assert_eq!(make_initials("Иржи Кропачек, Йозеф Новотный"), "И.К.,Й.Н.");
        assert_eq!(make_initials("österreich"), "Ö.");
    }
}
