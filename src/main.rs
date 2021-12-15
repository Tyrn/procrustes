#[macro_use]
extern crate lazy_static;

use alphanumeric_sort::sort_path_slice;
use clap::{App, AppSettings, Arg, ArgMatches};
use itertools::join;
use regex::Regex;
use std::{
    ffi::OsStr,
    fs, io,
    io::Write,
    path::{Path, PathBuf},
    process::exit,
    time::Instant,
};
use taglib;
use unicode_segmentation::UnicodeSegmentation;

const APP_DESCRIPTION: &str = "Procrustes a.k.a. Damastes \
    is a CLI utility for copying directories and subdirectories \
    containing supported audio files in sequence, naturally sorted. \
    The end result is a \"flattened\" copy of the source subtree. \"Flattened\" means \
    that only a namesake of the root source directory is created, where all the files get \
    copied to, names prefixed with a serial number. Tag \"Track Number\" \
    is set, tags \"Title\", \"Artist\", and \"Album\" can be replaced optionally. \
    The writing process is strictly sequential: either starting with the number one file, \
    or in the reverse order. This can be important for some mobile devices. \
    \n\nExamples; <src> as a directory: \
    \n\nrobinson-crusoe $ procrustes -va 'Daniel \"Goldeneye\" Defoe' -m 'Robinson Crusoe' . \
    /run/media/player \
    \n\n<src> as a single file: \
    \n\nlibrary $ procrustes -va 'Vladimir Nabokov' -u 'Ada' ada.ogg .";

const WARNING_ICON: &str = "\u{01f4a7}";
// const INVALID_ICON: &str = "\u{00274c}";
// const SUSPICIOUS_ICON: &str = "\u{002754}";
const DONE_ICON: &str = "\u{01f7e2}";
const COLUMN_ICON: &str = "\u{002714}";

lazy_static! {
    static ref ARGS: ArgMatches<'static> = retrieve_args();
//    static ref IS_ALBUM: bool = is_album_tag();
//    static ref ALBUM: &'static str = album_tag();
    static ref SRC: PathBuf = pval("src");
    static ref DST: PathBuf = executive_dst();
    static ref KNOWN_EXTENSIONS: [&'static str; 8] = [
        "MP3", "OGG", "M4A", "M4B", "OPUS", "WMA", "FLAC", "APE",
    ];
    static ref INITIALS: String = if flag("a") {
        make_initials(sval("a"))
    } else {
        "".to_string()
    };
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
            let src = pval("src");
            if src.is_file() {
                src.file_stem()
            } else {
                src.file_name()
            }
            .unwrap()
            .to_str()
            .unwrap()
            .to_string()
        }
    );
    if flag("p") {
        pval("dst-dir")
    } else {
        [pval("dst-dir"), PathBuf::from(base_dst)].iter().collect()
    }
}

// Cuts the artist snippet to build a directory or file name.
fn artist(forw_dash: bool) -> String {
    if flag("a") {
        if forw_dash {
            format!(" - {}", sval("a"))
        } else {
            format!("{} - ", sval("a"))
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
    Path::new(sval(name)).canonicalize().unwrap()
}

fn is_album_tag() -> bool {
    if flag("u") && !flag("m") {
        true
    } else {
        flag("m")
    }
}

fn album_tag() -> &'static str {
    if flag("u") && !flag("m") {
        sval("u")
    } else {
        sval("m")
    }
}

fn retrieve_args() -> ArgMatches<'static> {
    App::new("procrustes")
        .setting(AppSettings::ColoredHelp)
        .version("v1.0.3")
        .author("")
        .about(APP_DESCRIPTION)
        .arg(
            Arg::with_name("v")
                .short("v")
                .long("verbose")
                .help("Verbose output"),
        )
        .arg(
            Arg::with_name("d")
                .short("d")
                .long("drop-tracknumber")
                .help("Do not set track numbers"),
        )
        .arg(
            Arg::with_name("s")
                .short("s")
                .long("strip-decorations")
                .help("Strip file and directory name decorations"),
        )
        .arg(
            Arg::with_name("f")
                .short("f")
                .long("file-title")
                .help("Use file name for title tag"),
        )
        .arg(
            Arg::with_name("F")
                .short("F")
                .long("file-title-num")
                .help("Use numbered file name for title tag"),
        )
        .arg(
            Arg::with_name("x")
                .short("x")
                .long("sort-lex")
                .help("Sort files lexicographically"),
        )
        .arg(
            Arg::with_name("t")
                .short("t")
                .long("tree-dst")
                .help("Retain the tree structure of the source album at destination"),
        )
        .arg(
            Arg::with_name("p")
                .short("p")
                .long("drop-dst")
                .help("Do not create destination directory"),
        )
        .arg(
            Arg::with_name("r")
                .short("r")
                .long("reverse")
                .help("Copy files in reverse order (number one file is the last to be copied)"),
        )
        .arg(
            Arg::with_name("i")
                .short("i")
                .long("prepend-subdir-name")
                .help("Prepend current subdirectory name to a file name"),
        )
        .arg(
            Arg::with_name("w")
                .short("w")
                .long("overwrite")
                .help("Silently remove existing destination directory (not recommended)"),
        )
        .arg(
            Arg::with_name("y")
                .short("y")
                .long("dry-run")
                .help("Without actually copying the files (trumps -w, too)"),
        )
        .arg(
            Arg::with_name("e")
                .short("e")
                .long("file-type")
                .value_name("EXT")
                .help("Accept only audio files of the specified type")
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
                .long("artist")
                .value_name("ARTIST")
                .help("Artist tag")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("m")
                .short("m")
                .long("album")
                .value_name("ALBUM")
                .help("Album tag")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("src")
                .help("Source file or directory")
                .required(true)
                .index(1),
        )
        .arg(
            Arg::with_name("dst-dir")
                .help("Destination directory")
                .required(true)
                .index(2),
        )
        .get_matches()
}

fn check_args() {
    let (src, dst) = (pval("src"), pval("dst-dir"));

    if !src.exists() {
        println!(
            " {} Source directory \"{}\" is not there.",
            WARNING_ICON,
            src.display()
        );
        exit(0);
    }
    if !dst.exists() {
        println!(
            " {} Destination path \"{}\" is not there.",
            WARNING_ICON,
            dst.display()
        );
        exit(0);
    }
    if !flag("p") && !flag("y") {
        if DST.exists() {
            if flag("w") {
                fs::remove_dir_all(&DST.as_path()).expect(
                    format!(
                        " {} Failed to remove destination directory \"{}\".",
                        WARNING_ICON,
                        DST.display()
                    )
                    .as_str(),
                );
            } else {
                println!(
                    " {} Destination directory \"{}\" already exists.",
                    WARNING_ICON,
                    DST.display()
                );
                exit(0);
            }
        }
        fs::create_dir(&DST.as_path()).expect(
            format!(
                " {} Destination directory \"{}\" already exists!",
                WARNING_ICON,
                DST.display()
            )
            .as_str(),
        );
    }
}

fn tracks_count(dir: &Path) -> (usize, u64) {
    if dir.is_file() {
        if one_for_audiofile(dir) > 0 {
            return (1, dir.metadata().unwrap().len());
        }
        return (0, 0);
    }
    let mut sum = 0;
    let cnt = fs::read_dir(dir)
        .unwrap()
        .into_iter()
        .filter(|r| r.is_ok())
        .map(|r| {
            let p = r.unwrap().path();
            if p.is_dir() {
                let rv = tracks_count(&p);
                sum += rv.1;
                rv.0
            } else {
                if one_for_audiofile(&p) > 0 {
                    sum += &p.metadata().unwrap().len();
                    1
                } else {
                    0
                }
            }
        })
        .sum();
    (cnt, sum)
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
                one_for_audiofile(&r) > 0
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
    if dir.is_file() && one_for_audiofile(dir) > 0 {
        return (vec![], vec![dir.to_path_buf()]);
    }
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

fn copy_album(count: usize) {
    if count < 1 {
        println!("No audio files found at \"{}\"", SRC.display());
        exit(0);
    }
    let width = format!("{}", count).len();

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
                let name = format!(
                    "{}-{}{}.{}",
                    prefix,
                    sval("u"),
                    artist(true),
                    ext.to_str().unwrap()
                );
                PathBuf::from(name)
            } else {
                let fnm = src.file_name().unwrap();
                let name = format!("{}-{}", prefix, fnm.to_str().unwrap());
                PathBuf::from(name)
            }
        }
    };

    // Calculates destination for [src] file to be copied to and
    // makes a copy.
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

            // Calculates the contents for the title tag.
            let title = |s: &str| -> String {
                let stem = &src.file_stem().unwrap().to_str().unwrap();
                if flag("F") {
                    format!("{}>{}", ii, &stem)
                } else if flag("f") {
                    stem.to_string()
                } else {
                    format!("{} {}", ii, s)
                }
            };

            if !flag("d") {
                tag.set_track(ii as u32);
            }
            if flag("a") && is_album_tag() {
                tag.set_title(&title(&format!("{} - {}", INITIALS.as_str(), album_tag())));
                tag.set_artist(sval("a"));
                tag.set_album(album_tag());
            } else if flag("a") {
                tag.set_title(&title(sval("a")));
                tag.set_artist(sval("a"));
            } else if is_album_tag() {
                tag.set_title(&title(album_tag()));
                tag.set_album(album_tag());
            }

            tag_file.save();
        }

        if flag("v") {
            println!(
                "{:1$}/{2} {3} {4}",
                ii,
                width,
                count,
                COLUMN_ICON,
                &dst.to_str().unwrap()
            );
        } else {
            print!(".");
            io::stdout().flush().unwrap();
        }
    };

    if !flag("v") {
        print!("Starting ");
        io::stdout().flush().unwrap();
    }

    // Calculates file number.
    macro_rules! entry_num {
        ($i: expr) => {
            if flag("r") {
                count - $i
            } else {
                $i + 1
            }
        };
    }

    for (i, (src, step)) in traverse_dir(&SRC, [].to_vec()).enumerate() {
        copy(entry_num!(i), &src, &step);
    }
}

fn main() {
    check_args();
    let now = Instant::now();
    let (count, size) = tracks_count(&SRC.as_path());

    copy_album(count);

    println!(
        " {} Done ({}, {}; {:.1}s).",
        DONE_ICON,
        count,
        size,
        now.elapsed().as_secs_f64()
    );
}

/// Returns 1, if [path] has an audio file extension, otherwise 0.
///
#[allow(dead_code)]
fn one_for_audiofile_ext(path: &Path) -> usize {
    KNOWN_EXTENSIONS
        .iter()
        .any(|ext| has_ext_of(path.to_str().unwrap(), ext)) as usize
}

/// Returns 1, if [path] is a valid audio file, otherwise 0.
///
fn one_for_audiofile(path: &Path) -> usize {
    match taglib::File::new(path) {
        Err(_) => 0,
        Ok(v) => match v.tag() {
            Err(_) => 0,
            Ok(_) => 1,
        },
    }
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

/// Returns a comma-separated list of initials,
/// [authors] being a comma-separated list of full names.
///
fn make_initials(authors: &str) -> String {
    lazy_static! {
        static ref SPACE: Regex = Regex::new(r"[\s.]+").unwrap();
        static ref NICKNAME: Regex = Regex::new(r#""(?:\\.|[^"\\])*""#).unwrap();
        static ref NOBILIARY_PARTICLES: [&'static str; 32] = [
            "von", "фон", "van", "ван", "der", "дер", "til", "тиль", "zu", "цу", "af", "аф", "of",
            "из", "de", "де", "des", "дез", "del", "дель", "dos", "душ", "дос", "du", "дю", "la",
            "ла", "ля", "le", "ле", "haut", "от",
        ];
    }

    fn gv(s: &str) -> Vec<&str> {
        UnicodeSegmentation::graphemes(s, true).collect::<Vec<&str>>()
    }

    fn form_initial(name: &str) -> String {
        let cut: Vec<&str> = name.split("'").collect();

        if cut.len() > 1 && !cut[1].is_empty() {
            // Deal with '.
            if cut[1].chars().next().unwrap().is_lowercase() && !cut[0].is_empty() {
                return gv(cut[0])[0].to_uppercase();
            }
            return cut[0].to_owned() + "'" + &gv(cut[1])[0];
        }

        let v = gv(name);
        let mut v_iter = v.iter();

        if v.len() > 1 {
            // Deal with prefixes.
            let mut prefix: Vec<&str> = vec![*v_iter.next().unwrap()];
            for vch in v_iter {
                prefix.push(vch);
                if vch.chars().next().unwrap().is_uppercase() {
                    return prefix.concat();
                }
            }
        }

        if NOBILIARY_PARTICLES.iter().any(|&x| name == x) {
            return v[0].to_string();
        }

        v[0].to_uppercase()
    }

    join(
        NICKNAME
            .replace_all(authors, " ")
            .replace("\"", " ")
            .split(",")
            .filter(|author| author.replace(".", "").replace("-", "").trim() != "")
            .map(|author| {
                [
                    join(
                        author
                            .split("-")
                            .filter(|barrel| barrel.replace(".", "").trim() != "")
                            .map(|barrel| {
                                join(
                                    SPACE
                                        .split(barrel)
                                        .filter(|name| !name.is_empty())
                                        .map(|name| form_initial(name)),
                                    ".",
                                )
                            }),
                        "-",
                    ),
                    ".".to_string(),
                ]
                .concat()
            }),
        ",",
    )
}

#[cfg(test)]
mod test_main;
