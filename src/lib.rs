mod spinner;
use crate::spinner as spin;
use crate::spinner::Spinner;

mod icons;
use crate::icons::*;

mod tags;
use tags::{file_set_tags, is_audiofile};

use alphanumeric_sort::sort_path_slice;
use chrono::{DateTime, Utc};
use clap::{parser::ValueSource, Arg, ArgAction, ArgMatches, Command};
use const_format::{concatcp, formatcp};
use itertools::join;
use itertools::zip_eq;
use owo_colors::OwoColorize;
use regex::Regex;
use std::{
    cmp, env,
    ffi::OsStr,
    fs, io,
    io::Write,
    path::{Path, PathBuf, MAIN_SEPARATOR_STR},
    process::exit,
    sync::LazyLock,
    sync::OnceLock,
    time::Instant,
};
use tempfile::TempDir;
use unicode_segmentation::UnicodeSegmentation;

const APP_DESCRIPTION: &str = concatcp!(
    "Procrustes a.k.a. Damastes \
    is a CLI utility for copying directories and subdirectories \
    containing supported audio files in sequence, naturally sorted. \
    The end result is a \"flattened\" copy of the source subtree. \"Flattened\" means \
    that only a namesake of the root source directory is created, where all the files get \
    copied to, names prefixed with a serial number. Tag \"Track Number\" \
    is set, tags \"Title\", \"Artist\", and \"Album\" can be replaced optionally. \
    The writing process is strictly sequential: either starting with the number one file, \
    or in the reverse order. This can be important for some mobile devices. ",
    USE_ICON,
    " Really, ",
    PRAISE_ICON,
    " really useful options. ",
    SUSPICIOUS_ICON,
    " Suspicious media. \
    \n\nExamples; <src> as a directory: \
    \n\nrobinson-crusoe $ procrustes -va 'Daniel \"Goldeneye\" Defoe' -m 'Robinson Crusoe' . \
    /run/media/player \
    \n\n<src> as a single file: \
    \n\nlibrary $ procrustes -va 'Vladimir Nabokov' -u 'Ada' ada.ogg ."
);

#[derive(Debug)]
struct Context {
    tracks: u64,  // Audiofile count in the source tree
    width: usize, // Decimal digits of [tracks]
    bytes: u64,   // Sum of the file sizes of all the tracks
    dst: PathBuf, // Destination root directory
}

static CONTEXT: OnceLock<Context> = OnceLock::new();

fn ctx() -> &'static Context {
    CONTEXT.get().expect("CONTEXT not initialized")
}

static KNOWN_EXTENSIONS: [&str; 9] = [
    "MP3", "OGG", "M4A", "M4B", "OPUS", "WMA", "FLAC", "APE", "WAV",
];

static ARGS: LazyLock<ArgMatches> = LazyLock::new(args_retrieve);
static DST_DIR: LazyLock<PathBuf> = LazyLock::new(dst_calculate);
static IS_ARTIST: LazyLock<bool> = LazyLock::new(|| flag("a"));
static IS_UNIFIED: LazyLock<bool> = LazyLock::new(|| flag("u"));
static UNIFIED: LazyLock<String> = LazyLock::new(|| {
    if *IS_UNIFIED {
        sval("u").to_string()
    } else {
        "".to_string()
    }
});
static IS_ALBUM: LazyLock<bool> = LazyLock::new(|| {
    if *IS_UNIFIED && !flag("m") {
        true
    } else {
        flag("m")
    }
});
static ARTIST: LazyLock<String> = LazyLock::new(|| {
    if *IS_ARTIST {
        sval("a").to_string()
    } else {
        "".to_string()
    }
});
static ALBUM: LazyLock<String> = LazyLock::new(|| {
    if *IS_ALBUM {
        if *IS_UNIFIED && !flag("m") {
            UNIFIED.to_string()
        } else {
            sval("m").to_string()
        }
    } else {
        "".to_string()
    }
});

/// Holds the -v option count.
///
static VERBOSE_LEVEL: LazyLock<u8> = LazyLock::new(|| match ARGS.value_source("v") {
    Some(ValueSource::CommandLine) => ARGS.get_count("v"),
    _ => 0,
});

/// Returns true, if the [name] option is present
/// on the command line.
///
fn flag(name: &str) -> bool {
    matches!(ARGS.value_source(name), Some(ValueSource::CommandLine))
}

/// Returns the string value, associated with the [name] option.
/// Defined, if flag(name) is true.
fn sval(name: &str) -> &'static str {
    ARGS.get_one::<String>(name)
        .map(|s| s.as_str())
        .unwrap_or("NULL_STR")
}

/// Returns the PathBuf value, associated with the [name] option.
/// Defined, if flag(name) is true.
///
fn pval(name: &str) -> PathBuf {
    Path::new(sval(name)).canonicalize().unwrap_or_else(|_| {
        panic!(
            "{}File or directory \"{}\" does not exist.{}",
            BDELIM_ICON,
            sval(name),
            BDELIM_ICON
        )
    })
}

/// Sets up command line parser, and gets the command line
/// options and arguments.
///
fn args_retrieve() -> ArgMatches {
    Command::new("procrustes")
        .version("v1.0.3")
        .author("")
        .about(APP_DESCRIPTION)
        .arg(
            Arg::new("v")
                .short('v')
                .long("verbose")
                .help(formatcp!("{USE_ICON} Verbose output"))
                .action(ArgAction::Count)
        )
        .arg(
            Arg::new("d")
                .short('d')
                .long("drop-tracknumber")
                .help("Do not set track numbers")
                .action(ArgAction::SetTrue)
        )
        .arg(
            Arg::new("s")
                .short('s')
                .long("strip-decorations")
                .help("Strip file and directory name decorations")
                .action(ArgAction::SetTrue)
        )
        .arg(
            Arg::new("f")
                .short('f')
                .long("file-title")
                .help("Use file name for title tag")
                .action(ArgAction::SetTrue)
        )
        .arg(
            Arg::new("F")
                .short('F')
                .long("file-title-num")
                .help("Use numbered file name for title tag")
                .action(ArgAction::SetTrue)
        )
        .arg(
            Arg::new("x")
                .short('x')
                .long("sort-lex")
                .help("Sort files lexicographically")
                .action(ArgAction::SetTrue)
        )
        .arg(
            Arg::new("t")
                .short('t')
                .long("tree-dst")
                .help("Retain the tree structure of the source album at destination")
                .action(ArgAction::SetTrue)
        )
        .arg(
            Arg::new("p")
                .short('p')
                .long("drop-dst")
                .help("Do not create destination directory")
                .action(ArgAction::SetTrue)
        )
        .arg(
            Arg::new("r")
                .short('r')
                .long("reverse")
                .help("Copy files in reverse order (number one file is the last to be copied)")
                .action(ArgAction::SetTrue)
        )
        .arg(
            Arg::new("i")
                .short('i')
                .long("prepend-subdir-name")
                .help("Prepend current subdirectory name to a file name")
                .action(ArgAction::SetTrue)
        )
        .arg(
            Arg::new("c")
                .short('c')
                .long("count")
                .help("Just count the files")
                .action(ArgAction::SetTrue)
        )
        .arg(
            Arg::new("w")
                .short('w')
                .long("overwrite")
                .help("Silently remove existing destination directory (not recommended)")
                .action(ArgAction::SetTrue)
        )
        .arg(
            Arg::new("y")
                .short('y')
                .long("dry-run")
                .help("Without actually copying the files (trumps -w, too)")
                .action(ArgAction::SetTrue)
        )
        .arg(
            Arg::new("e")
                .short('e')
                .long("file-type")
                .value_name("EXT")
                .help("Accept only audio files of the specified type (e.g. -e ogg, or even -e '*kb.mp3')")
                .action(ArgAction::Set)
        )
        .arg(
            Arg::new("u")
                .short('u')
                .long("unified-name")
                .value_name("UNAME")
                .help(formatcp!("{USE_ICON}{PRAISE_ICON} UNAME for everything unspecified"))
                .action(ArgAction::Set)
        )
        .arg(
            Arg::new("b")
                .short('b')
                .long("album-num")
                .value_name("ALBUM_NUM")
                .help("0..99; prepend ALBUM_NUM to the destination root directory name")
                .value_parser(clap::value_parser!(i64))
                .action(ArgAction::Set)
        )
        .arg(
            Arg::new("a")
                .short('a')
                .long("artist")
                .value_name("ARTIST")
                .help(formatcp!("{USE_ICON}{PRAISE_ICON} Artist tag"))
                .action(ArgAction::Set)
        )
        .arg(
            Arg::new("m")
                .short('m')
                .long("album")
                .value_name("ALBUM")
                .help(formatcp!("{USE_ICON} Album tag"))
                .action(ArgAction::Set)
        )
        .arg(
            Arg::new("src")
                .help("Source file or directory")
                .required(true)
                .index(1),
        )
        .arg(
            Arg::new("dst-dir")
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
        .filter(|r| r.is_ok())
        .map(|r| r.unwrap().path())
        .filter(|r| if folders { r.is_dir() } else { is_audiofile(r) })
        .collect())
}

#[allow(dead_code)]
/// Returns a vector of the directories and files inside [dir].
///
fn dir_offspring(dir: &Path) -> Result<Vec<PathBuf>, io::Error> {
    fs::read_dir(dir)?
        .map(|x| x.map(|entry| entry.path()))
        .collect()
}

/// Returns sorted vectors of directories and audiofiles inside [dir].
///
fn dir_groom(dir: &Path) -> (Vec<PathBuf>, Vec<PathBuf>) {
    fn sort_lex(dirs: &mut [PathBuf], files: &mut [PathBuf]) {
        dirs.sort_unstable();
        files.sort_unstable();
    }
    fn sort_naturally(dirs: &mut [PathBuf], files: &mut [PathBuf]) {
        sort_path_slice(dirs);
        sort_path_slice(files);
    }

    fn reverse(dirs: &mut [PathBuf], files: &mut [PathBuf]) {
        dirs.reverse();
        files.reverse();
    }
    fn reverse_nop(_dirs: &mut [PathBuf], _files: &mut [PathBuf]) {}

    type Sort = fn(&mut [PathBuf], &mut [PathBuf]);
    type Reverse = fn(&mut [PathBuf], &mut [PathBuf]);

    static SORT: LazyLock<Sort> =
        LazyLock::new(|| if flag("x") { sort_lex } else { sort_naturally });
    static REVERSE: LazyLock<Reverse> =
        LazyLock::new(|| if flag("r") { reverse } else { reverse_nop });

    if dir.is_file() && is_audiofile(dir) {
        return (vec![], vec![dir.to_path_buf()]);
    }
    let mut dirs = fs_entries(dir, true).unwrap();
    let mut files = fs_entries(dir, false).unwrap();

    SORT(&mut dirs, &mut files);
    REVERSE(&mut dirs, &mut files);

    (dirs, files)
}

type WalkItem = (PathBuf, Vec<PathBuf>);
type WalkIterator = Box<dyn Iterator<Item = WalkItem>>;
type WalkStepDownFn = dyn Fn(PathBuf) -> WalkIterator;
type WalkMakeItemFn = dyn Fn(PathBuf) -> WalkItem;

/// Walks down the (source) [dir] tree, accumulating [step_down] on each recursion level.
/// Item is a tuple of
/// (audiofile, Vec<subdirectory (to be created at destination/to make it possible)>).
///
fn dir_walk(dir: &Path, step_down: Vec<PathBuf>) -> WalkIterator {
    fn stream_forward(
        dirs: Vec<PathBuf>,
        files: Vec<PathBuf>,
        walk: Box<WalkStepDownFn>,
        item: Box<WalkMakeItemFn>,
    ) -> WalkIterator {
        Box::new(
            dirs.into_iter()
                .flat_map(walk)
                .chain(files.into_iter().map(item)),
        )
    }
    fn stream_backward(
        dirs: Vec<PathBuf>,
        files: Vec<PathBuf>,
        walk: Box<WalkStepDownFn>,
        item: Box<WalkMakeItemFn>,
    ) -> WalkIterator {
        Box::new(
            files
                .into_iter()
                .map(item)
                .chain(dirs.into_iter().flat_map(walk)),
        )
    }
    type WalkStreamFn =
        fn(Vec<PathBuf>, Vec<PathBuf>, Box<WalkStepDownFn>, Box<WalkMakeItemFn>) -> WalkIterator;

    static STREAM_FILES: LazyLock<WalkStreamFn> = LazyLock::new(|| {
        if flag("r") {
            stream_backward
        } else {
            stream_forward
        }
    });

    let (dirs, files) = dir_groom(dir);
    let step = step_down.clone();

    let walk = move |d: PathBuf| {
        let mut step = step_down.clone();
        step.push(PathBuf::from(d.file_name().unwrap()));
        dir_walk(&d, step)
    };

    let item = move |f: PathBuf| (f, step.clone());

    STREAM_FILES(dirs, files, Box::new(walk), Box::new(item))
}

/// Copies [src] to [dst], makes panic sensible.
///
fn file_copy(src: &Path, dst: &Path) {
    fs::copy(src, dst).unwrap_or_else(|_| {
        panic!(
            "{}Error while copying \"{}\" to \"{}\".{}",
            BDELIM_ICON,
            src.to_str().unwrap(),
            dst.to_str().unwrap(),
            BDELIM_ICON
        )
    });
}

#[allow(dead_code)]
/// Copies [src] to [dst], sets tags to [dst].
///
fn file_copy_and_set_tags(ii: u64, src: &Path, dst: &Path) {
    file_copy(src, dst);
    file_set_tags(ii, src, dst);
}

#[allow(dead_code)]
/// Copies [src] to [dst], sets tags using a temporary file.
///
fn file_copy_and_set_tags_via_tmp(ii: u64, src: &Path, dst: &Path) {
    let tmp_dir = TempDir::new().unwrap(); // Keep it!
    let tmp = tmp_dir.path().join(format!(
        "tmpaudio.{}",
        src.extension().unwrap().to_str().unwrap()
    ));

    file_copy(src, &tmp);
    file_set_tags(ii, src, &tmp);
    file_copy(&tmp, dst);

    fs::remove_file(&tmp).unwrap_or_else(|_| {
        panic!(
            "{}Error while deleting \"{}\" file.{}",
            BDELIM_ICON,
            tmp.to_str().unwrap(),
            BDELIM_ICON
        )
    });
}

/// Checks the source validity, and its compatibility with the destination.
///
fn src_check() -> (Vec<String>, PathBuf) {
    let src = pval("src");
    let mut log = Vec::<String>::new();

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
    (log, src.to_path_buf())
}

/// Returns Artist, nicely shaped to be a part of a directory/file name.
///
fn artist_part(forw_dash: bool) -> String {
    if *IS_ARTIST {
        if forw_dash {
            format!(" - {}", *ARTIST)
        } else {
            format!("{} - ", *ARTIST)
        }
    } else {
        "".to_string()
    }
}

/// Returns the destination directory path, calculated according to options.
/// The destination directory is calculated, not created here.
///
fn dst_calculate() -> PathBuf {
    let prefix = if let Some(n) = ARGS.get_one::<i64>("b") {
        format!("{:02}-", n)
    } else {
        "".to_string()
    };
    let base_dst = format!(
        "{}{}",
        prefix,
        if *IS_UNIFIED {
            format!("{}{}", artist_part(false), *UNIFIED)
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

/// Creates destination directory according to options, if
/// necessary and possible.
///
fn dst_create() -> PathBuf {
    if !flag("p") && !flag("y") {
        if DST_DIR.exists() {
            if flag("w") {
                fs::remove_dir_all(DST_DIR.as_path()).unwrap_or_else(|_| {
                    panic!(
                        "{}Failed to remove destination directory \"{}\".{}",
                        BDELIM_ICON,
                        DST_DIR.display(),
                        BDELIM_ICON
                    )
                });
            } else {
                println!(
                    " {} Destination directory \"{}\" already exists.",
                    WARNING_ICON,
                    DST_DIR.display()
                );
                exit(1);
            }
        }
        fs::create_dir(DST_DIR.as_path()).unwrap_or_else(|_| {
            panic!(
                "{}Destination directory \"{}\" already exists!{}",
                BDELIM_ICON,
                DST_DIR.display(),
                BDELIM_ICON
            )
        });
    }
    DST_DIR.to_path_buf()
}

/// Extracts file name from the [src] track (number [ii])
/// and makes it pretty, if necessary.
///
fn track_decorate(ii: u64, src: &Path, step: &[PathBuf]) -> PathBuf {
    fn prefix_subdir_make(ii: u64, step: &[PathBuf]) -> String {
        if !step.is_empty() {
            format!(
                "{:01$}-[{2}]",
                ii,
                ctx().width,
                join(step.iter().map(|p| p.to_str().unwrap()), "][")
            )
        } else {
            format!("{:01$}", ii, ctx().width)
        }
    }
    fn prefix_make(ii: u64, _step: &[PathBuf]) -> String {
        format!("{:01$}", ii, ctx().width)
    }
    fn decorate_unified(ii: u64, src: &Path, step: &[PathBuf]) -> PathBuf {
        PathBuf::from(format!(
            "{}-{}{}.{}",
            PREFIX_MAKE(ii, step),
            *UNIFIED,
            artist_part(true),
            src.extension().unwrap().to_str().unwrap()
        ))
    }
    fn decorate(ii: u64, src: &Path, step: &[PathBuf]) -> PathBuf {
        PathBuf::from(format!(
            "{}-{}",
            PREFIX_MAKE(ii, step),
            src.file_name().unwrap().to_str().unwrap()
        ))
    }
    fn decorate_nop(_ii: u64, src: &Path, _step: &[PathBuf]) -> PathBuf {
        PathBuf::from(src.file_name().unwrap())
    }

    type PrefixMake = fn(u64, &[PathBuf]) -> String;
    type Decorate = fn(u64, &Path, &[PathBuf]) -> PathBuf;

    static PREFIX_MAKE: LazyLock<PrefixMake> = LazyLock::new(|| {
        if flag("i") && !flag("t") {
            prefix_subdir_make
        } else {
            prefix_make
        }
    });
    static DECORATE: LazyLock<Decorate> = LazyLock::new(|| {
        if flag("s") && flag("t") {
            decorate_nop
        } else if *IS_UNIFIED {
            decorate_unified
        } else {
            decorate
        }
    });

    DECORATE(ii, src, step)
}

/// Calculates destination, complete with the file name,
/// for the [src_file] track to be copied to and
/// makes the copy of the valid track (number [ii]).
///
fn track_copy(
    ii: u64,
    src_file: &Path,
    step: &[PathBuf], // All the subdirectories to be created below [dst].
    log: &mut Vec<String>,
) {
    fn step_flat_collect(_step: &[PathBuf]) -> PathBuf {
        PathBuf::new()
    }
    fn step_collect(step: &[PathBuf]) -> PathBuf {
        step.iter().collect()
    }

    fn step_nop_create_dir(_dst: &Path, _step: &Path) {}

    fn step_create_dir(dst: &Path, step: &Path) {
        let dst_dir = dst.join(step);
        fs::create_dir_all(&dst_dir).unwrap_or_else(|_| {
            panic!(
                "{}Error while creating \"{}\" directory.{}",
                BDELIM_ICON,
                dst_dir.to_str().unwrap(),
                BDELIM_ICON
            )
        });
    }

    #[allow(clippy::ptr_arg)]
    fn file_nop_copytags(_ii: u64, _src: &Path, _dst: &Path, _log: &mut Vec<String>) -> u64 {
        0
    }

    fn file_copytags(ii: u64, src: &Path, dst: &Path, log: &mut Vec<String>) -> u64 {
        let mut dst_bytes: u64 = 0;

        if dst.is_file() {
            log.push(format!(
                " {} File \"{}\" already copied. Review your options.",
                WARNING_ICON,
                dst.file_name().unwrap().to_str().unwrap()
            ));
        } else {
            file_copy_and_set_tags_via_tmp(ii, src, dst);
            dst_bytes = dst.metadata().unwrap().len();
        }
        dst_bytes
    }

    fn out_track_verbose(ii: u64, path: &Path, dst_bytes: u64, src_bytes: u64) {
        print!(
            "{:1$}{2}{3}  {4}",
            ii,
            ctx().width,
            COLUMN_ICON,
            ctx().tracks,
            path.strip_prefix(DST_DIR.as_path())
                .unwrap_or(path)
                .display()
        );
        if dst_bytes != src_bytes {
            if dst_bytes == 0 {
                print!("    {}", human_fine(src_bytes).cyan()); // Dry run
            } else {
                let growth = dst_bytes as i64 - src_bytes as i64;
                let s = format!("{:+}", growth);

                print!("    {}", s.cyan());
            }
        }
        println!();
    }
    fn out_track_terse(_ii: u64, _path: &Path, _dst_bytes: u64, _src_bytes: u64) {
        print!(".");
        io::stdout().flush().unwrap();
    }

    type StepCollect = fn(&[PathBuf]) -> PathBuf;
    type StepCreateDir = fn(&Path, &Path);
    type FileCopyTags = fn(u64, &Path, &Path, &mut Vec<String>) -> u64;
    type OutTrack = fn(u64, &Path, u64, u64);

    static STEP_COLLECT: LazyLock<StepCollect> = LazyLock::new(|| {
        if flag("t") {
            step_collect
        } else {
            step_flat_collect
        }
    });
    static STEP_CREATE_DIR: LazyLock<StepCreateDir> = LazyLock::new(|| {
        if flag("t") && !flag("y") {
            step_create_dir
        } else {
            step_nop_create_dir
        }
    });
    static FILE_COPYTAGS: LazyLock<FileCopyTags> = LazyLock::new(|| {
        if flag("y") {
            file_nop_copytags
        } else {
            file_copytags
        }
    });
    static OUT_TRACK: LazyLock<OutTrack> = LazyLock::new(|| {
        if flag("v") {
            out_track_verbose
        } else {
            out_track_terse
        }
    });

    let stride: PathBuf = STEP_COLLECT(step);
    STEP_CREATE_DIR(&ctx().dst, &stride);
    let dst_file = &ctx()
        .dst
        .join(&stride)
        .join(track_decorate(ii, src_file, step));

    OUT_TRACK(
        ii,
        dst_file,
        FILE_COPYTAGS(ii, src_file, dst_file, log),
        src_file.metadata().unwrap().len(),
    );
}

/// Copies all the valid tracks to their destination, according to
/// the options and GlobalState.
///
fn album_copy(now: &Instant, src: &Path, log: &mut Vec<String>) {
    fn show_dst() {
        let s = format!("{}{}", DST_DIR.display(), MAIN_SEPARATOR_STR);
        println!(" >> {} >>", s.cyan());
    }

    fn out_start_terse() {
        show_dst();
        print!(" {} ", START_ICON);
        io::stdout().flush().unwrap();
    }
    fn out_nop() {
        show_dst();
    }

    fn out_tail_terse() {
        println!(" {}", STOP_ICON);
        show_dst();
    }

    fn out_done(time_elapsed: f64) {
        println!(
            " {} Done ({}, {}; {:.1}s).",
            DONE_ICON,
            ctx().tracks,
            human_fine(ctx().bytes),
            time_elapsed,
        );
    }

    fn tracks_range(range: u64) -> Box<dyn Iterator<Item = u64>> {
        Box::new(1_u64..=range)
    }
    fn tracks_range_rev(range: u64) -> Box<dyn Iterator<Item = u64>> {
        Box::new((1_u64..=range).rev())
    }

    static OUT_START: LazyLock<fn()> =
        LazyLock::new(|| if flag("v") { out_nop } else { out_start_terse });
    static OUT_TAIL: LazyLock<fn()> =
        LazyLock::new(|| if flag("v") { out_nop } else { out_tail_terse });
    static OUT_DONE: LazyLock<fn(f64)> = LazyLock::new(|| out_done);

    /// Iterator over track indices.
    type TrackIter = Box<dyn Iterator<Item = u64>>;

    /// A function producing an iterator of track indices from a count.
    type RangeFn = fn(u64) -> TrackIter;

    static RANGE: LazyLock<RangeFn> = LazyLock::new(|| {
        if flag("r") {
            tracks_range_rev
        } else {
            tracks_range
        }
    });

    if ctx().tracks < 1 {
        println!(
            " {} No audio files found at \"{}\"",
            WARNING_ICON,
            src.display()
        );
        exit(1);
    }

    OUT_START();

    for (i, (src, step)) in zip_eq(RANGE(ctx().tracks), dir_walk(src, [].to_vec())) {
        track_copy(i, &src, &step, log);
    }

    OUT_TAIL();
    OUT_DONE(now.elapsed().as_secs_f64());
}

/// Returns the statistics of all the audiofiles in [dir] and its subdirectories
/// in the form of a tuple:
/// (
/// Full count of the files with common extensions, which failed to open (suspicious),
/// Sum of all the sizes of the suspicious audiofiles/tracks (suspicious bytes)
/// Full count of the valid audiofiles (tracks),
/// Sum of all the sizes of the valid audiofiles (bytes)
/// )
///
fn tracks_count(
    dir: &Path,
    spinner: &mut dyn Spinner,
    log: &mut Vec<String>,
) -> (u64, u64, u64, u64) {
    fn log_name_v(p: &Path) -> String {
        let (icon, stamp) = match p.metadata().unwrap().created() {
            Ok(date) => (BDELIM_ICON, DateTime::<Utc>::from(date)),
            Err(_) => match p.metadata().unwrap().modified() {
                Ok(date) => (ODELIM_ICON, DateTime::<Utc>::from(date)),
                Err(_) => (
                    INVALID_ICON,
                    DateTime::<Utc>::from(std::time::SystemTime::now()),
                ),
            },
        };
        format!(
            "{}{} {}  {} {}",
            &stamp.date_naive().to_string()[..10],
            icon,
            p.strip_prefix(env::current_dir().unwrap())
                .unwrap()
                .to_str()
                .unwrap(),
            COLUMN_ICON,
            human_fine(p.metadata().unwrap().len()),
        )
    }
    fn log_name(p: &Path) -> String {
        p.file_name().unwrap().to_str().unwrap().to_string()
    }

    if dir.is_file() {
        if is_audiofile(dir) {
            return (0, 0, 1, dir.metadata().unwrap().len());
        } else if is_pattern_ok(dir) && is_audiofile_ext(dir) {
            log.push(format!(" {} {}", SUSPICIOUS_ICON, log_name_v(dir)));
            return (1, dir.metadata().unwrap().len(), 0, 0);
        }
        return (0, 0, 0, 0);
    }

    let mut bytes = 0;
    let mut suspicious = 0;
    let mut suspicious_bytes = 0;

    let tracks = fs::read_dir(dir)
        .unwrap()
        .filter(|r| r.is_ok())
        .map(|r| {
            let p = r.unwrap().path();
            if p.is_dir() {
                let (sc, sb, tc, tb) = tracks_count(&p, spinner, log);
                suspicious += sc;
                suspicious_bytes += sb;
                bytes += tb;
                tc
            } else if is_audiofile(&p) {
                bytes += &p.metadata().unwrap().len();
                spinner.message(log_name(&p));
                1
            } else {
                if is_pattern_ok(&p) && is_audiofile_ext(&p) {
                    suspicious += 1;
                    suspicious_bytes += &p.metadata().unwrap().len();
                    log.push(format!(" {} {}", SUSPICIOUS_ICON, log_name_v(&p)))
                }
                0
            }
        })
        .sum();

    (suspicious, suspicious_bytes, tracks, bytes)
}

pub fn app() {
    LazyLock::force(&ARGS);
    // let _ = *ARGS; // This magic works just as nice.

    let mut log: Vec<String> = Vec::new();
    let (mut src_check_log, src) = src_check();

    let now = Instant::now();
    let mut spinner = spin::PrettySpinner::new();

    let (suspicious_total, suspicious_bytes_total, tracks_total, bytes_total) =
        tracks_count(src.as_path(), &mut spinner, &mut log);

    spinner.stop();
    log.sort_unstable(); // Suspicious files only, sorting them by date.
    log.append(&mut src_check_log);

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
        CONTEXT
            .set(Context {
                tracks: tracks_total,
                width: format!("{}", tracks_total).len(),
                bytes: bytes_total,
                dst: dst_create(),
            })
            .expect("CONTEXT already set");

        album_copy(&now, &src, &mut log);

        // Second pass through the source done, all the tracks, if any, copied to destination.
    }
    for s in log {
        println!("{}", s);
    }
    if suspicious_total > 0 {
        println!(
            " {} Suspicious, skipped: {} file(s); Volume: {}",
            RSUSP_ICON,
            suspicious_total,
            human_fine(suspicious_bytes_total)
        );
    }

    // Final report done.
}

/// Returns a human readable string representation of [bytes], nicely rounded.
///
fn human_fine(bytes: u64) -> String {
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
fn str_shrink(s: &str, limit: usize) -> String {
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

/// Returns true, if [path] satisfies file-type (-e) CLI suggestion,
/// otherwise false.
/// If the file type is not supplied, returns true.
///
fn is_pattern_ok(path: &Path) -> bool {
    fn is_regex() -> bool {
        if flag("e") {
            let e = sval("e");
            e.contains('*') || e.contains('[') || e.contains(']') || e.contains('?')
        } else {
            false
        }
    }
    fn always_true(_path: &Path) -> bool {
        true
    }
    fn is_ext_matching(path: &Path) -> bool {
        has_ext_of(path.to_str().unwrap(), &EXT)
    }
    fn is_pattern_matching(path: &Path) -> bool {
        (*PATTERN).matches(path.file_name().unwrap().to_str().unwrap())
    }

    static EXT: LazyLock<String> = LazyLock::new(|| {
        if flag("e") {
            sval("e").to_string()
        } else {
            "".to_string()
        }
    });
    static PATTERN: LazyLock<glob::Pattern> = LazyLock::new(|| {
        if is_regex() {
            glob::Pattern::new(sval("e")).unwrap()
        } else {
            glob::Pattern::new("*").unwrap()
        }
    });
    static IS_PATTERN_OK: LazyLock<fn(&Path) -> bool> = LazyLock::new(|| {
        if is_regex() {
            is_pattern_matching
        } else if flag("e") {
            is_ext_matching
        } else {
            always_true
        }
    });

    (*IS_PATTERN_OK)(path)
}

/// Returns true, if [path] has an audio file extension, otherwise false.
///
fn is_audiofile_ext(path: &Path) -> bool {
    KNOWN_EXTENSIONS
        .iter()
        .any(|ext| has_ext_of(path.to_str().unwrap(), ext))
}

fn has_ext_of(path: &str, ext: &str) -> bool {
    let p = path.to_uppercase();
    let e = ext.to_uppercase().replace('.', "");
    Path::new(&p).extension() == Some(OsStr::new(&e))
}

#[allow(dead_code)]
/// Returns a vector of integer numbers, embedded in [s].
///
fn str_strip_numbers(s: &str) -> Vec<i64> {
    static RE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"\d+").unwrap());
    // iterate over all matches
    RE.find_iter(s)
        .filter_map(|digits| digits.as_str().parse().ok()) // Filter out numbers out of ixx range.
        .collect()
}

/// Returns a comma-separated list of initials,
/// [authors] being a comma-separated list of full names.
///
fn initials(authors: &str) -> String {
    static SPACE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"[\s.]+").unwrap());
    static NICKNAME: LazyLock<Regex> =
        LazyLock::new(|| Regex::new(r#""(?:\\.|[^"\\])*""#).unwrap());
    static NOBILIARY_PARTICLES: [&str; 47] = [
        "von", "фон", "van", "ван", "der", "дер", "til", "тиль", "zu", "цу", "zum", "цум", "zur",
        "цур", "af", "аф", "of", "из", "da", "да", "de", "де", "des", "дез", "del", "дель", "den",
        "ден", "di", "ди", "dos", "душ", "дос", "du", "дю", "la", "ла", "ля", "le", "ле", "las",
        "лас", "los", "лос", "haut", "от", "the",
    ];

    fn gv(s: &str) -> Vec<&str> {
        UnicodeSegmentation::graphemes(s, true).collect()
    }

    /// Converts [name] to its initial. Mostly by keeping the first character
    /// and dropping the rest; deals with special cases, too. See the unit test.
    ///
    fn initial(name: &str) -> String {
        let cut: Vec<&str> = name.split('\'').collect();

        if cut.len() > 1 && !cut[1].is_empty() {
            // Deal with '.
            if cut[1].chars().next().unwrap().is_lowercase() && !cut[0].is_empty() {
                return gv(cut[0])[0].to_uppercase();
            }
            return cut[0].to_owned() + "'" + gv(cut[1])[0];
        }

        let v = gv(name);
        let mut v_iter = v.iter();

        if v.len() > 1 {
            // Deal with prefixes.
            match name {
                "Старший" => return "Ст".to_string(),
                "Младший" => return "Мл".to_string(),
                "Ст" | "ст" | "Sr" | "Мл" | "мл" | "Jr" => return name.to_string(),
                _ => (),
            };
            let mut prefix: Vec<&str> = vec![*v_iter.next().unwrap()];
            for vch in v_iter {
                prefix.push(vch);
                if vch.chars().next().unwrap().is_uppercase() {
                    return prefix.concat();
                }
            }
        }

        if v[0].chars().next().unwrap().is_lowercase() && NOBILIARY_PARTICLES.contains(&name) {
            return v[0].to_string();
        }

        v[0].to_uppercase()
    }

    join(
        NICKNAME
            .replace_all(authors, " ")
            .replace('"', " ")
            .split(',')
            .filter(|author| author.replace(['.', '-'], "").trim() != "")
            .map(|author| {
                [
                    join(
                        author
                            .split('-')
                            .filter(|barrel| barrel.replace('.', "").trim() != "")
                            .map(|barrel| {
                                join(
                                    SPACE
                                        .split(barrel)
                                        .filter(|name| !name.is_empty())
                                        .map(initial),
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
