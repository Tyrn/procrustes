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
use walkdir::WalkDir;
use taglib;

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

// Calculates the destination directory according to options.
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

#[allow(dead_code)]
fn is_album_tag() -> bool {
    if flag("u") && !flag("g") {
        true
    } else {
        flag("g")
    }
}

#[allow(dead_code)]
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
            Arg::with_name("w")
                .short("w")
                .long("overwrite")
                .help("silently remove existing destination directory (not recommended)"),
        )
        .arg(
            Arg::with_name("y")
                .short("y")
                .long("dry-run")
                .help("without actually copying the files"),
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
    if !flag("p") && !flag("y") {
        if DST.exists() {
            if flag("w") {
                fs::remove_dir_all(&DST.as_path()).expect(
                    format!(
                        "Failed to remove destination directory \"{}\".",
                        DST.display()
                    )
                    .as_str(),
                );
            } else {
                println!(
                    "Destination directory \"{}\" already exists.",
                    DST.display()
                );
                exit(0);
            }
        }
        fs::create_dir(&DST.as_path()).expect(
            format!(
                "Destination directory \"{}\" already exists!",
                DST.display()
            )
            .as_str(),
        );
    }
}

fn audiofiles_count(dir: &Path) -> usize {
    let mut count = 0;

    for i in WalkDir::new(dir) {
        let entry = i.unwrap();
        let path = entry.path();
        if path.is_file() && is_audiofile(path.to_str().unwrap()) {
            count = count + 1;
        }
    }
    count
}

fn fs_entries(dir: &Path, folders: bool) -> Result<Vec<PathBuf>, io::Error> {
    Ok(fs::read_dir(dir)?
        .into_iter()
        .filter(|r| r.is_ok())
        .map(|r| r.unwrap().path())
        .filter(|r| {
            if folders {
                r.is_dir()
            } else {
                r.is_file() && is_audiofile(r.to_str().unwrap())
            }
        })
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

fn traverse_dir(
    src_dir: &PathBuf,
    dst_step: Vec<PathBuf>,
) -> Box<dyn Iterator<Item = (PathBuf, Vec<PathBuf>)>> {
    let (dirs, files) = groom(src_dir);
    let destination_step = dst_step.clone();

    let traverse = move |d: PathBuf| {
        let mut step = dst_step.clone();
        step.push(PathBuf::from(d.file_name().unwrap()));
        traverse_dir(&d, step)
    };
    let handle = move |f: PathBuf| (f, destination_step.clone());
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
    check_args();

    let count = audiofiles_count(&SRC.as_path());
    if count < 1 {
        println!("No audio files found at \"{}\"", SRC.display());
        exit(0);
    }
    let width = format!("{}", count).len();

    let n = |i| if flag("r") { count - i } else { i + 1 };

    // Extracts file name from [src] and makes it pretty, if necessary.
    let decor = |ii, src: &PathBuf, step: &Vec<PathBuf>| -> PathBuf {
        if flag("s") && flag("t") {
            PathBuf::from(src.file_name().unwrap())
        } else {
            let prefix = if flag("i") && !flag("t") {
                if step.len() > 0 {
                    let lines = step.iter().map(|p| p.to_str().unwrap());
                    let chain = join(lines, "-");
                    format!("{:01$}-[{2}]", ii, width, chain)
                } else {
                    format!("{:01$}", ii, width)
                }
            } else {
                format!("{:01$}", ii, width)
            };

            if flag("u") {
                let ext = src.extension().unwrap();
                let name = format!("{}-{}.{}", prefix, sval("u"), ext.to_str().unwrap());
                PathBuf::from(name)
            } else {
                let fnm = src.file_name().unwrap();
                let name = format!("{}-{}", prefix, fnm.to_str().unwrap());
                PathBuf::from(name)
            }
        }
    };

    let copy = |ii, src: &PathBuf, step: &Vec<PathBuf>| {
        let file_name = decor(ii, src, step);
        let depth: PathBuf = if flag("t") {
            step.iter().collect()
        } else {
            PathBuf::new()
        };
        if flag("t") && !flag("y") {
            let dst_dir = DST.join(&depth);
            fs::create_dir_all(&dst_dir).expect(
                format!(
                    "Error while creating \"{}\" directory.",
                    &dst_dir.to_str().unwrap()
                )
                .as_str(),
            );
        }
        let dst = DST.join(&depth).join(&file_name);

        // All the copying and tagging happens here.
        if !flag("y") {
            fs::copy(&src, &dst).expect(
                format!("Error while copying \"{}\" file.", &dst.to_str().unwrap()).as_str(),
            );
            let tag_file = taglib::File::new(&dst).expect(
                format!(
                    "Error while opening \"{}\" for tagging.",
                    &dst.to_str().unwrap()
                )
                .as_str(),
            );
            let mut tag = tag_file.tag().expect("No tagging data.");
            tag.set_album("ZZZ");
            tag_file.save();
        }

        if flag("v") {
            println!("{:1$}/{2} {3}", ii, width, count, &dst.to_str().unwrap());
        } else {
            print!(".");
        }
    };

    if !flag("v") {
        print!("Starting ");
    }

    for (i, (src, step)) in traverse_dir(&SRC, [].to_vec()).enumerate() {
        copy(n(i), &src, &step);
    }

    if !flag("v") {
        println!(" Done ({}).", count);
    }
}

fn main() {
    copy_album();
}

/// Returns true, if [path] is a recognized audio file.
///
fn is_audiofile(path: &str) -> bool {
    let exts = ["MP3", "M4A", "M4B", "OGG", "WMA", "FLAC"];
    exts.iter().any(|ext| has_ext_of(path, ext))
}

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
    fn checking_pad() {
        assert_eq!(format!("{:01$}", 1, 4), "0001");
    }

    #[test]
    fn checking_audiofile() {
        assert_eq!(is_audiofile("/alfa/bra.vo/charlie.ogg"), true);
        assert_eq!(is_audiofile("/alfa/bra.vo/charlie.MP3"), true);
        assert_eq!(is_audiofile("/alfa/bra.vo/charlie.pdf"), false);
        assert_eq!(is_audiofile("/alfa/bra.vo/charlie"), false);
    }

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
