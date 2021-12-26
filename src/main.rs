#[macro_use]
extern crate lazy_static;

mod spinner;
use crate::spinner as spin;
use crate::spinner::Spinner;

use alphanumeric_sort::sort_path_slice;
use clap::{App, AppSettings, Arg, ArgMatches};
use glob;
use itertools::join;
use regex::Regex;
use std::{
    cmp,
    ffi::OsStr,
    fs, io,
    io::Write,
    path::{Path, PathBuf},
    process::exit,
    time::Instant,
};
use taglib;
use tempfile::TempDir;
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
    \u{002754} Suspicious media. \
    \n\nExamples; <src> as a directory: \
    \n\nrobinson-crusoe $ procrustes -va 'Daniel \"Goldeneye\" Defoe' -m 'Robinson Crusoe' . \
    /run/media/player \
    \n\n<src> as a single file: \
    \n\nlibrary $ procrustes -va 'Vladimir Nabokov' -u 'Ada' ada.ogg .";

// const INVALID_ICON: &str = "\u{00274c}";
const WARNING_ICON: &str = "\u{01f4a7}";
const BDELIM_ICON: &str = "\u{01f539}";
const RSUSP_ICON: &str = "\u{002753}";
const SUSPICIOUS_ICON: &str = "\u{002754}";
const DONE_ICON: &str = "\u{01f7e2}";
const COLUMN_ICON: &str = "\u{002714}";
const LINK_ICON: &str = "\u{0026a1}";

lazy_static! {
    static ref ARGS: ArgMatches<'static> = args_retrieve();
    static ref DST_DIR: PathBuf = dst_calculate();
    static ref KNOWN_EXTENSIONS: [&'static str; 9] =
        ["MP3", "OGG", "M4A", "M4B", "OPUS", "WMA", "FLAC", "APE", "WAV",];
    static ref ARTIST: String = if flag("a") {
        sval("a").to_string()
    } else {
        "".to_string()
    };
    static ref ALBUM: String = if is_album_tag() {
        album_tag().to_string()
    } else {
        "".to_string()
    };
    static ref INITIALS: String = if flag("a") {
        initials(&ARTIST)
    } else {
        "".to_string()
    };
    static ref TAG_SET_TRACK: fn(&mut taglib::Tag, usize) = if flag("d") {
        tag_nop_track
    } else {
        tag_set_track
    };
    static ref TITLE_TAIL: String = if flag("a") && is_album_tag() {
        format!("{} - {}", INITIALS.as_str(), ALBUM.as_str())
    } else if flag("a") {
        ARTIST.to_string()
    } else if is_album_tag() {
        ALBUM.to_string()
    } else {
        "".to_string()
    };
    static ref TITLE_COMPOSE: fn(usize, &PathBuf) -> String = if flag("F") {
        title_fi
    } else if flag("f") {
        title_f
    } else {
        title_i
    };
    static ref TAG_SET_ALL: fn(&mut taglib::Tag, usize, &PathBuf) = if flag("a") && is_album_tag() {
        tag_set_artist_album
    } else if flag("a") {
        tag_set_artist
    } else if is_album_tag() {
        tag_set_album
    } else {
        tag_nop_all
    };
    static ref OUT_START: fn() = if flag("v") {
        out_start_nop
    } else {
        out_start_terse
    };
    static ref OUT_TRACK: fn(usize, usize, u64, &str, u64, u64) = if flag("v") {
        out_track_v
    } else {
        out_track_terse
    };
    static ref OUT_DONE: fn(u64, u64, f64) = out_done;
}

/// Returns the destination directory, calculated according to options.
///
fn dst_calculate() -> PathBuf {
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

/// Returns Artist, nicely shaped to be a part of a directory/file name.
///
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

/// Returns true, if the [name] option is present
/// on the command line.
///
fn flag(name: &str) -> bool {
    if ARGS.occurrences_of(name) > 0 {
        true
    } else {
        false
    }
}

/// Returns the string value, associated with the [name] option.
/// Defined, if flag(name) is true.
///
fn sval(name: &str) -> &str {
    ARGS.value_of(name).unwrap_or("NULL_STR")
}

/// Returns the integer value, associated with the [name] option.
/// Defined, if flag(name) is true.
///
fn ival(name: &str) -> i64 {
    ARGS.value_of(name)
        .unwrap_or("NULL_INT")
        .parse()
        .expect("Option value must be a valid number!")
}

/// Returns the PathBuf value, associated with the [name] option.
/// Defined, if flag(name) is true.
///
fn pval(name: &str) -> PathBuf {
    Path::new(sval(name)).canonicalize().expect(&format!(
        "{}File or directory \"{}\" does not exist.{}",
        BDELIM_ICON,
        sval(name),
        BDELIM_ICON
    ))
}

/// Returns true, if album tag is present on the command line.
///
fn is_album_tag() -> bool {
    if flag("u") && !flag("m") {
        true
    } else {
        flag("m")
    }
}

/// Returns album tag value.
/// Defined, if is_album_tag() is true.
///
fn album_tag() -> &'static str {
    if flag("u") && !flag("m") {
        sval("u")
    } else {
        sval("m")
    }
}

// Output callbacks.

fn out_start_terse() {
    print!("Starting ");
    io::stdout().flush().unwrap();
}

fn out_start_nop() {}

fn out_track_v(
    ii: usize,
    width: usize,
    tracks_total: u64,
    path: &str,
    dst_bytes: u64,
    src_bytes: u64,
) {
    print!(
        "{:1$}/{2} {3} {4}",
        ii, width, tracks_total, COLUMN_ICON, path,
    );
    if dst_bytes != src_bytes {
        if dst_bytes == 0 {
            print!("  {} {}", COLUMN_ICON, human_fine(src_bytes));
        } else {
            let growth = dst_bytes as i64 - src_bytes as i64;

            print!("  {} {:+}", COLUMN_ICON, growth);
        }
    }
    println!("");
}

fn out_track_terse(
    _ii: usize,
    _width: usize,
    _tracks_total: u64,
    _path: &str,
    _dst_bytes: u64,
    _src_bytes: u64,
) {
    print!(".");
    io::stdout().flush().unwrap();
}

fn out_done(tracks_total: u64, bytes_total: u64, time_elapsed: f64) {
    println!(
        " {} Done ({}, {}; {:.1}s).",
        DONE_ICON,
        tracks_total,
        human_fine(bytes_total),
        time_elapsed,
    );
}

// Title tag calculation callbacks.

fn title_fi(ii: usize, src: &PathBuf) -> String {
    let stem = &src.file_stem().unwrap().to_str().unwrap();

    format!("{}>{}", ii, &stem)
}

fn title_f(_ii: usize, src: &PathBuf) -> String {
    let stem = &src.file_stem().unwrap().to_str().unwrap();

    stem.to_string()
}

fn title_i(ii: usize, _src: &PathBuf) -> String {
    format!("{} {}", ii, TITLE_TAIL.to_string())
}

// Tag setting callbacks, see global static.

fn tag_set_track(tag: &mut taglib::Tag, ii: usize) {
    tag.set_track(ii as u32);
}

fn tag_nop_track(_tag: &mut taglib::Tag, _ii: usize) {}

fn tag_set_artist_album(tag: &mut taglib::Tag, ii: usize, src: &PathBuf) {
    tag.set_title(&TITLE_COMPOSE(ii, &src));
    tag.set_artist(&ARTIST);
    tag.set_album(&ALBUM);
}

fn tag_set_artist(tag: &mut taglib::Tag, ii: usize, src: &PathBuf) {
    tag.set_title(&TITLE_COMPOSE(ii, &src));
    tag.set_artist(&ARTIST);
}

fn tag_set_album(tag: &mut taglib::Tag, ii: usize, src: &PathBuf) {
    tag.set_title(&TITLE_COMPOSE(ii, &src));
    tag.set_album(&ALBUM);
}

fn tag_nop_all(_tag: &mut taglib::Tag, _ii: usize, _src: &PathBuf) {}

/// Sets up command line parser, and gets the command line
/// options and arguments.
///
fn args_retrieve() -> ArgMatches<'static> {
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
            Arg::with_name("c")
                .short("c")
                .long("count")
                .help("Just count the files"),
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
                .help("Accept only audio files of the specified type (e.g. -e ogg, or even -e '*kb.mp3')")
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

/// Returns a vector of [dir] subdirectories, if [folders] is true,
/// otherwise returns a vector of the audiofiles inside the [dir] directories.
///
fn fs_entries(dir: &Path, folders: bool) -> Result<Vec<PathBuf>, io::Error> {
    Ok(fs::read_dir(dir)?
        .into_iter()
        .filter(|r| r.is_ok())
        .map(|r| r.unwrap().path())
        .filter(|r| {
            if folders {
                r.is_dir()
            } else {
                is_audiofile(&r)
            }
        })
        .collect())
}

#[allow(dead_code)]
/// Returns a vector of the directories and files inside [dir].
///
fn dir_offspring(dir: &Path) -> Result<Vec<PathBuf>, io::Error> {
    fs::read_dir(dir)?
        .into_iter()
        .map(|x| x.map(|entry| entry.path()))
        .collect()
}

/// Returns sorted vectors of directories and audiofiles inside [dir].
///
fn dir_groom(dir: &Path) -> (Vec<PathBuf>, Vec<PathBuf>) {
    if dir.is_file() && is_audiofile(dir) {
        return (vec![], vec![dir.to_path_buf()]);
    }
    let mut dirs = fs_entries(dir, true).unwrap();
    let mut files = fs_entries(dir, false).unwrap();
    if flag("x") {
        // Sort lexicographically.
        dirs.sort_unstable();
        files.sort_unstable();
    } else {
        // Sort naturally.
        sort_path_slice(&mut dirs);
        sort_path_slice(&mut files);
    }
    if flag("r") {
        // Reverse sorting order.
        dirs.reverse();
        files.reverse();
    }
    (dirs, files)
}

/// Walks down the (source) [dir] tree, accumulating [step_down] on each recursion level.
/// Item is a tuple of
/// (audiofile, Vec<subdirectory (to be created at destination/to make it possible)>).
///
fn dir_walk(
    dir: &PathBuf,
    step_down: Vec<PathBuf>,
) -> Box<dyn Iterator<Item = (PathBuf, Vec<PathBuf>)>> {
    let (dirs, files) = dir_groom(dir);
    let step = step_down.clone();

    let walk = move |d: PathBuf| {
        let mut step = step_down.clone();
        step.push(PathBuf::from(d.file_name().unwrap()));
        dir_walk(&d, step)
    };
    let item = move |f: PathBuf| (f, step.clone());
    if flag("r") {
        Box::new(
            files
                .into_iter()
                .map(item)
                .chain(dirs.into_iter().flat_map(walk)),
        )
    } else {
        Box::new(
            dirs.into_iter()
                .flat_map(walk)
                .chain(files.into_iter().map(item)),
        )
    }
}

/// Copies [src] to [dst], makes panic sensible.
///
fn file_copy(src: &PathBuf, dst: &PathBuf) {
    fs::copy(&src, &dst).expect(
        format!(
            "{}Error while copying \"{}\" to \"{}\".{}",
            BDELIM_ICON,
            &src.to_str().unwrap(),
            &dst.to_str().unwrap(),
            BDELIM_ICON,
        )
        .as_str(),
    );
}

/// Sets tags to [dst] audio file, using [ii] and [src] name in the title tag
/// composition.
///
fn file_set_tags(ii: usize, src: &PathBuf, dst: &PathBuf) {
    let tag_file = taglib::File::new(&dst).expect(
        format!(
            "{}Error while opening \"{}\" for tagging.{}",
            BDELIM_ICON,
            &dst.to_str().unwrap(),
            BDELIM_ICON,
        )
        .as_str(),
    );
    let mut tag = tag_file
        .tag()
        .expect(format!("{}No tagging data.{}", BDELIM_ICON, BDELIM_ICON,).as_str());

    TAG_SET_TRACK(&mut tag, ii);
    TAG_SET_ALL(&mut tag, ii, &src);

    tag_file.save();
}

#[allow(dead_code)]
/// Copies [src] to [dst], sets tags to [dst].
///
fn file_copy_and_set_tags(ii: usize, src: &PathBuf, dst: &PathBuf) {
    file_copy(&src, &dst);
    file_set_tags(ii, &src, &dst);
}

#[allow(dead_code)]
/// Copies [src] to [dst], sets tags using a temporary file.
///
fn file_copy_and_set_tags_via_tmp(ii: usize, src: &PathBuf, dst: &PathBuf) {
    let tmp_dir = TempDir::new().unwrap(); // Keep it!
    let tmp = tmp_dir.path().join(format!(
        "tmpaudio.{}",
        &src.extension().unwrap().to_str().unwrap()
    ));

    file_copy(&src, &tmp);
    file_set_tags(ii, &src, &tmp);
    file_copy(&tmp, &dst);

    fs::remove_file(&tmp).expect(
        format!(
            "{}Error while deleting \"{}\" file.{}",
            BDELIM_ICON,
            &tmp.to_str().unwrap(),
            BDELIM_ICON,
        )
        .as_str(),
    );
}

/// Checks the source validity, and its compatibility with the destination.
///
fn src_check(log: &mut Vec<String>) -> PathBuf {
    let src = pval("src");

    if !flag("c") && src.is_dir() && DST_DIR.starts_with(&src) {
        let dst_msg = format!(
            " {} Target directory \"{}\"",
            WARNING_ICON,
            DST_DIR.display()
        );
        let src_msg = format!(" {} is inside source \"{}\"", WARNING_ICON, src.display());
        if flag("y") {
            log.push(dst_msg);
            log.push(src_msg);
            log.push(format!(" {} It won't run.", WARNING_ICON));
        } else {
            println!("{}", dst_msg);
            println!("{}", src_msg);
            println!(" {} No go.", WARNING_ICON);
            exit(1);
        }
    }
    src.to_path_buf()
}

/// Creates destination boiderplate according to options, if possible.
///
fn dst_create() -> PathBuf {
    if !flag("p") && !flag("y") {
        if DST_DIR.exists() {
            if flag("w") {
                fs::remove_dir_all(&DST_DIR.as_path()).expect(
                    format!(
                        "{}Failed to remove destination directory \"{}\".{}",
                        BDELIM_ICON,
                        DST_DIR.display(),
                        BDELIM_ICON,
                    )
                    .as_str(),
                );
            } else {
                println!(
                    " {} Destination directory \"{}\" already exists.",
                    WARNING_ICON,
                    DST_DIR.display()
                );
                exit(1);
            }
        }
        fs::create_dir(&DST_DIR.as_path()).expect(
            format!(
                "{}Destination directory \"{}\" already exists!{}",
                BDELIM_ICON,
                DST_DIR.display(),
                BDELIM_ICON,
            )
            .as_str(),
        );
    }
    DST_DIR.to_path_buf()
}

/// Extracts file name from the [src] track number [ii]
/// and makes it pretty, if necessary.
fn track_decorate(ii: usize, src: &PathBuf, step: &Vec<PathBuf>, width: usize) -> PathBuf {
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
}

/// Calculates destination for the [src] track to be copied to and
/// makes the copy of the valid track number [ii].
///
fn track_copy(
    ii: usize,
    src: &PathBuf,
    step: &Vec<PathBuf>,
    dst: &PathBuf,
    width: usize,
    tracks_total: u64,
    log: &mut Vec<String>,
) {
    let file_name = track_decorate(ii, src, step, width);
    let depth: PathBuf = if flag("t") {
        step.iter().collect()
    } else {
        PathBuf::new()
    };
    if flag("t") && !flag("y") {
        let dst_dir = dst.join(&depth);
        fs::create_dir_all(&dst_dir).expect(
            format!(
                "{}Error while creating \"{}\" directory.{}",
                BDELIM_ICON,
                &dst_dir.to_str().unwrap(),
                BDELIM_ICON,
            )
            .as_str(),
        );
    }

    let dst = dst.join(&depth).join(&file_name);

    let src_bytes: u64 = src.metadata().unwrap().len();
    let mut dst_bytes: u64 = 0;

    // All the copying and tagging happens here.
    if !flag("y") {
        if dst.is_file() {
            log.push(format!(
                " {} File \"{}\" already copied. Review your options.",
                WARNING_ICON,
                &dst.file_name().unwrap().to_str().unwrap()
            ));
        } else {
            file_copy_and_set_tags_via_tmp(ii, &src, &dst);
            dst_bytes = dst.metadata().unwrap().len();
        }
    }

    OUT_TRACK(
        ii,
        width,
        tracks_total,
        &dst.to_str().unwrap(),
        dst_bytes,
        src_bytes,
    );
}

/// Copies all the valid tracks to their destination, according to
/// the options and GlobalState.
///
fn album_copy(
    now: &Instant,
    src: &PathBuf,
    dst: &PathBuf,
    width: usize,
    tracks_total: u64,
    bytes_total: u64,
    log: &mut Vec<String>,
) {
    if tracks_total < 1 {
        println!(
            " {} No audio files found at \"{}\"",
            WARNING_ICON,
            src.display()
        );
        exit(1);
    }

    OUT_START();

    // Calculates file number.
    macro_rules! entry_num {
        ($i: expr) => {
            if flag("r") {
                tracks_total as usize - $i
            } else {
                $i + 1
            }
        };
    }

    let mut tracks_total_check: u64 = 0;

    for (i, (src, step)) in dir_walk(src, [].to_vec()).enumerate() {
        track_copy(entry_num!(i), &src, &step, dst, width, tracks_total, log);
        tracks_total_check += 1;
    }

    if tracks_total_check != tracks_total {
        panic!(
            "{}Fatal error: tracks discovered on first pass: {}; on secons pass: {}.{}",
            BDELIM_ICON, tracks_total, tracks_total_check, BDELIM_ICON,
        );
    }

    OUT_DONE(tracks_total, bytes_total, now.elapsed().as_secs_f64());
}

/// Returns full recursive count of audiofiles in [dir],
/// and the sum of their sizes.
///
/// Sets self.suspicious_total.
///
fn tracks_count(dir: &Path, spinner: &mut dyn Spinner, log: &mut Vec<String>) -> (u64, u64, u64) {
    if dir.is_file() {
        if is_audiofile(dir) {
            return (0, 1, dir.metadata().unwrap().len());
        }
        return (0, 0, 0);
    }

    let mut bytes = 0;
    let mut suspicious = 0;

    let tracks = fs::read_dir(dir)
        .unwrap()
        .into_iter()
        .filter(|r| r.is_ok())
        .map(|r| {
            let p = r.unwrap().path();
            if p.is_dir() {
                let count = tracks_count(&p, spinner, log);
                suspicious += count.0;
                bytes += count.2;
                count.1
            } else {
                let file_name = String::from(&p.file_name().unwrap().to_str().unwrap().to_string());

                if is_audiofile(&p) {
                    spinner.message(file_name);
                    bytes += &p.metadata().unwrap().len();
                    1
                } else {
                    if is_pattern_ok(&p) && is_audiofile_ext(&p) {
                        suspicious += 1;
                        log.push(format!(" {} {}", SUSPICIOUS_ICON, file_name))
                    }
                    0
                }
            }
        })
        .sum();

    (suspicious, tracks, bytes)
}

fn main() {
    lazy_static::initialize(&ARGS); // Make sure arguments are handled at this point.
                                    // let _ = *ARGS; // This magic works just as nice.

    let mut log: Vec<String> = Vec::new();
    let src = src_check(&mut log);

    let now = Instant::now();
    let mut spinner = spin::DaddySpinner::new();

    let count = tracks_count(src.as_path(), &mut spinner, &mut log);
    spinner.stop();

    let suspicious_total = count.0;
    let tracks_total = count.1;
    let bytes_total = count.2;

    // First pass through the source done, statistics collected.

    if flag("c") {
        print!(
            " {} Valid: {} file(s); Volume: {}",
            if tracks_total > 0 {
                DONE_ICON
            } else {
                WARNING_ICON
            },
            tracks_total,
            human_fine(bytes_total)
        );
        if tracks_total > 1 {
            print!("; Average: {}", human_fine(bytes_total / tracks_total));
        }
        println!("; Time: {:.1}s", now.elapsed().as_secs_f64())

        // Statistics reported, nothing else to be done.
    } else {
        album_copy(
            &now,
            &src,
            &dst_create(),
            format!("{}", tracks_total).len(),
            tracks_total,
            bytes_total,
            &mut log,
        );

        // Second pass through the source done, all the tracks, if any, copied to destination.
    }
    for s in log {
        println!("{}", s);
    }
    if suspicious_total > 0 {
        println!(
            " {} Suspicious, skipped: {} file(s)",
            RSUSP_ICON, suspicious_total
        );
    }

    // Final report done.
}

/// Returns a human readable string representation of [bytes], nicely rounded.
///
fn human_fine(bytes: u64) -> String {
    lazy_static! {
        static ref UNIT_LIST: [(&'static str, i32); 6] = [
            ("", 0),
            ("kB", 0),
            ("MB", 1),
            ("GB", 2),
            ("TB", 2),
            ("PB", 2),
        ];
    }
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
fn str_shrink(s: &str, limit: usize) -> String {
    let s: Vec<char> = s.chars().collect();
    let limit = cmp::max(10, limit);
    if s.len() > limit {
        let (head, tail) = s.split_at(s.len() / 2);
        let (hh, _) = head.split_at(limit / 2);
        let (_, tt) = tail.split_at(tail.len() - limit / 2);
        return format!(
            "{} {} {}",
            hh.into_iter().collect::<String>().trim(),
            LINK_ICON,
            tt.into_iter().collect::<String>().trim()
        );
    }
    return s.into_iter().collect();
}

/// Returns true, if [path] satisfies file-type (-e) CLI suggestion,
/// otherwise false.
/// If the file type is not supplied, returns true.
///
fn is_pattern_ok(path: &Path) -> bool {
    if flag("e") {
        let e = sval("e");
        if e.contains("*") || e.contains("[") || e.contains("?") {
            let pattern = glob::Pattern::new(e).unwrap();
            pattern.matches(path.file_name().unwrap().to_str().unwrap())
        } else {
            has_ext_of(path.to_str().unwrap(), e)
        }
    } else {
        true
    }
}

/// Returns true, if [path] has an audio file extension, otherwise false.
///
fn is_audiofile_ext(path: &Path) -> bool {
    KNOWN_EXTENSIONS
        .iter()
        .any(|ext| has_ext_of(path.to_str().unwrap(), ext))
}

/// Returns true, if [path] is a valid audio file, otherwise false.
///
fn is_audiofile(path: &Path) -> bool {
    if is_pattern_ok(path) {
        match taglib::File::new(path) {
            Err(_) => false,
            Ok(v) => match v.tag() {
                Err(_) => false,
                Ok(_) => true,
            },
        }
    } else {
        false
    }
}

fn has_ext_of(path: &str, ext: &str) -> bool {
    let p = path.to_uppercase();
    let e = ext.to_uppercase().replace(".", "");
    Path::new(&p).extension() == Some(OsStr::new(&e))
}

#[allow(dead_code)]
/// Returns a vector of integer numbers, embedded in [s].
///
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
fn initials(authors: &str) -> String {
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
        UnicodeSegmentation::graphemes(s, true).collect()
    }

    /// Converts [name] to its initial. Mostly by keeping the first character
    /// and dropping the rest; deals with special cases, too. See the unit test.
    ///
    fn initial(name: &str) -> String {
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
                                        .map(|name| initial(name)),
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
