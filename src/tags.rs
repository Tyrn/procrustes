//! Audio file detection and tag editing.
//!
//! Two backends are available and are both compiled in by default:
//!
//!   * `backend-taglib` — uses the TagLib C++ library
//!   * `backend-lofty`  — uses the pure-Rust `lofty` crate
//!
//! The runtime flag `-l` / `--alt-backend` selects between them.
//! If only one feature is compiled in, the dispatcher falls back
//! to it regardless of the flag.
//!
//! Build with a single backend (lean build):
//!
//! ```text
//! cargo build --no-default-features --features backend-taglib
//! cargo build --no-default-features --features backend-lofty
//! ```

use crate::{flag, VERBOSE_LEVEL};
use std::path::Path;
use std::sync::LazyLock;

// ---------------------------------------------------------------------
// Backend availability guards
// ---------------------------------------------------------------------

#[cfg(not(any(feature = "backend-taglib", feature = "backend-lofty")))]
compile_error!("at least one of `backend-taglib` or `backend-lofty` must be enabled");

// ---------------------------------------------------------------------
// Shared helpers
// ---------------------------------------------------------------------

/// Suppresses TagLib/Lofty warnings unless the user asked for them.
fn quiet_if_not_verbose() -> Option<gag::Gag> {
    (*VERBOSE_LEVEL < 2).then(|| gag::Gag::stderr().unwrap())
}

// ---------------------------------------------------------------------
// Shared tag-composition state
//
// Compiled regardless of which backends are present.
// ---------------------------------------------------------------------

use crate::{initials::initials, ALBUM, ARTIST, IS_ALBUM, IS_ARTIST};

static INITIALS: LazyLock<String> = LazyLock::new(|| {
    if *IS_ARTIST {
        initials(&ARTIST)
    } else {
        String::new()
    }
});

static TITLE_TAIL: LazyLock<String> = LazyLock::new(|| {
    if *IS_ARTIST && *IS_ALBUM {
        format!("{} - {}", INITIALS.as_str(), ALBUM.as_str())
    } else if *IS_ARTIST {
        ARTIST.to_string()
    } else if *IS_ALBUM {
        ALBUM.to_string()
    } else {
        String::new()
    }
});

type TitleCompose = fn(u64, &Path) -> String;

fn title_fi(ii: u64, src: &Path) -> String {
    let stem = src.file_stem().unwrap().to_str().unwrap();
    format!("{}>{}", ii, stem)
}

fn title_f(_ii: u64, src: &Path) -> String {
    let stem = src.file_stem().unwrap().to_str().unwrap();
    stem.to_string()
}

fn title_i(ii: u64, _src: &Path) -> String {
    format!("{} {}", ii, *TITLE_TAIL)
}

static TITLE_COMPOSE: LazyLock<TitleCompose> = LazyLock::new(|| {
    if flag("F") {
        title_fi
    } else if flag("f") {
        title_f
    } else {
        title_i
    }
});

// ---------------------------------------------------------------------
// taglib backend
// ---------------------------------------------------------------------

#[cfg(feature = "backend-taglib")]
/// Returns true if `path` is a valid audio file according to `TagLib`.
pub fn is_audiofile_taglib(path: &Path) -> bool {
    use crate::is_pattern_ok;

    let _silence = quiet_if_not_verbose();

    if !is_pattern_ok(path) {
        return false;
    }
    match taglib::File::new(path) {
        Err(_) => false,
        Ok(v) => v.tag().is_ok(),
    }
}

#[cfg(feature = "backend-taglib")]
/// Sets tags to `dst` using `TagLib`, composing the title from `ii` and `src`.
pub fn file_set_tags_taglib(ii: u64, src: &Path, dst: &Path) {
    use crate::BDELIM_ICON;

    let _silence = quiet_if_not_verbose();

    // ---- taglib-specific dispatch tables ----

    fn tag_set_track_number(tag: &mut taglib::Tag, ii: u64) {
        tag.set_track(ii as u32);
    }

    fn tag_nop_track_number(_tag: &mut taglib::Tag, _ii: u64) {}

    fn tag_set_artist_album(tag: &mut taglib::Tag, ii: u64, src: &Path) {
        tag.set_title(&TITLE_COMPOSE(ii, src));
        tag.set_artist(&ARTIST);
        tag.set_album(&ALBUM);
    }

    fn tag_set_artist(tag: &mut taglib::Tag, ii: u64, src: &Path) {
        tag.set_title(&TITLE_COMPOSE(ii, src));
        tag.set_artist(&ARTIST);
    }

    fn tag_set_album(tag: &mut taglib::Tag, ii: u64, src: &Path) {
        tag.set_title(&TITLE_COMPOSE(ii, src));
        tag.set_album(&ALBUM);
    }

    fn tag_nop_all(_tag: &mut taglib::Tag, _ii: u64, _src: &Path) {}

    type TagSetTrackNumber = fn(&mut taglib::Tag, u64);
    type TagSetTheRest = fn(&mut taglib::Tag, u64, &Path);

    static TAG_SET_TRACK_NUMBER: LazyLock<TagSetTrackNumber> = LazyLock::new(|| {
        if flag("d") {
            tag_nop_track_number
        } else {
            tag_set_track_number
        }
    });

    static TAG_SET_THE_REST: LazyLock<TagSetTheRest> = LazyLock::new(|| {
        if *IS_ARTIST && *IS_ALBUM {
            tag_set_artist_album
        } else if *IS_ARTIST {
            tag_set_artist
        } else if *IS_ALBUM {
            tag_set_album
        } else {
            tag_nop_all
        }
    });

    // ---- open, tag, save ----

    let tag_file = taglib::File::new(dst).unwrap_or_else(|_| {
        panic!(
            "{}Error while opening \"{}\" for tagging.{}",
            BDELIM_ICON,
            dst.to_str().unwrap(),
            BDELIM_ICON
        )
    });

    let mut tag = tag_file
        .tag()
        .unwrap_or_else(|_| panic!("{}No tagging data.{}", BDELIM_ICON, BDELIM_ICON));

    TAG_SET_TRACK_NUMBER(&mut tag, ii);
    TAG_SET_THE_REST(&mut tag, ii, src);

    let _ = tag_file.save();
}

// ---------------------------------------------------------------------
// lofty backend
// ---------------------------------------------------------------------

/// Parsing mode shared by the Lofty functions.
///
/// `BestAttempt` and `Relaxed` behave identically in practice for
/// real-world libraries; `Strict` rejects roughly 30% of a typical
/// audiobook collection.
#[cfg(feature = "backend-lofty")]
const PARSING_MODE: lofty::config::ParsingMode = lofty::config::ParsingMode::Relaxed;

#[cfg(feature = "backend-lofty")]
/// Returns true if `path` is a valid audio file according to `Lofty`.
pub fn is_audiofile_lofty(path: &Path) -> bool {
    use crate::is_pattern_ok;
    use lofty::config::ParseOptions;
    use lofty::probe::Probe;

    let _silence = quiet_if_not_verbose();

    if !is_pattern_ok(path) {
        return false;
    }

    let options = ParseOptions::new().parsing_mode(PARSING_MODE);

    Probe::open(path)
        .and_then(|probe| probe.options(options).read())
        .is_ok()
}

#[cfg(feature = "backend-lofty")]
/// Sets tags to `dst` using `Lofty`, composing the title from `ii` and `src`.
pub fn file_set_tags_lofty(ii: u64, src: &Path, dst: &Path) {
    use crate::BDELIM_ICON;

    use lofty::config::{ParseOptions, WriteOptions};
    use lofty::file::AudioFile;
    use lofty::prelude::{Accessor, TaggedFileExt};
    use lofty::probe::Probe;
    use lofty::tag::{Tag, TagType};

    let _silence = quiet_if_not_verbose();

    // ---- open the destination (relaxed, no panic) ----

    let options = ParseOptions::new().parsing_mode(PARSING_MODE);

    let mut tagged = match Probe::open(dst).and_then(|probe| probe.options(options).read()) {
        Ok(t) => t,
        Err(_) => {
            eprintln!(
                "{}Skipped \"{}\": Lofty could not read it for tagging.{}",
                BDELIM_ICON,
                dst.display(),
                BDELIM_ICON
            );
            return;
        }
    };

    // ---- compose the field values once ----

    let title = TITLE_COMPOSE(ii, src);
    let track = if flag("d") { None } else { Some(ii as u32) };
    let artist = if *IS_ARTIST {
        Some(ARTIST.clone())
    } else {
        None
    };
    let album = if *IS_ALBUM { Some(ALBUM.clone()) } else { None };

    let apply = |tag: &mut Tag| {
        tag.set_title(title.clone());
        if let Some(t) = track {
            tag.set_track(t);
        }
        if let Some(ref a) = artist {
            tag.set_artist(a.clone());
        }
        if let Some(ref a) = album {
            tag.set_album(a.clone());
        }
    };

    // ---- primary tag ----

    let tag_type = tagged.primary_tag_type();
    if tagged.primary_tag_mut().is_none() {
        tagged.insert_tag(Tag::new(tag_type));
    }

    match tagged.primary_tag_mut() {
        Some(primary) => apply(primary),
        None => {
            eprintln!(
                "{}Skipped \"{}\": no primary tag could be created.{}",
                BDELIM_ICON,
                dst.display(),
                BDELIM_ICON
            );
            return;
        }
    }

    // ---- additional ID3v1 tag (only meaningful for MP3) ----

    if tag_type == TagType::Id3v2 {
        if tagged.tag_mut(TagType::Id3v1).is_none() {
            tagged.insert_tag(Tag::new(TagType::Id3v1));
        }
        if let Some(id3v1) = tagged.tag_mut(TagType::Id3v1) {
            apply(id3v1);
        }
    }

    // ---- write back ----

    if tagged.save_to_path(dst, WriteOptions::default()).is_err() {
        eprintln!(
            "{}Failed to save tags to \"{}\".{}",
            BDELIM_ICON,
            dst.display(),
            BDELIM_ICON
        );
    }
}

// ---------------------------------------------------------------------
// Runtime dispatcher
//
// `-l` / `--alt-backend` selects Lofty; otherwise TagLib is used.
// If only one backend is compiled in, the dispatcher uses it and
// ignores the flag.
//
// The choice is pinned into a static function pointer via LazyLock,
// the same way the tag-composition and track-decoration dispatch
// tables work elsewhere in the crate.
// ---------------------------------------------------------------------

type IsAudiofile = fn(&Path) -> bool;
type FileSetTags = fn(u64, &Path, &Path);

// ---- both backends compiled in ----

#[cfg(all(feature = "backend-taglib", feature = "backend-lofty"))]
static IS_AUDIOFILE: LazyLock<IsAudiofile> = LazyLock::new(|| {
    if flag("l") {
        is_audiofile_lofty
    } else {
        is_audiofile_taglib
    }
});

#[cfg(all(feature = "backend-taglib", feature = "backend-lofty"))]
static FILE_SET_TAGS: LazyLock<FileSetTags> = LazyLock::new(|| {
    if flag("l") {
        file_set_tags_lofty
    } else {
        file_set_tags_taglib
    }
});

// ---- only taglib compiled in ----

#[cfg(all(feature = "backend-taglib", not(feature = "backend-lofty")))]
static IS_AUDIOFILE: LazyLock<IsAudiofile> = LazyLock::new(|| is_audiofile_taglib);

#[cfg(all(feature = "backend-taglib", not(feature = "backend-lofty")))]
static FILE_SET_TAGS: LazyLock<FileSetTags> = LazyLock::new(|| file_set_tags_taglib);

// ---- only lofty compiled in ----

#[cfg(all(feature = "backend-lofty", not(feature = "backend-taglib")))]
static IS_AUDIOFILE: LazyLock<IsAudiofile> = LazyLock::new(|| is_audiofile_lofty);

#[cfg(all(feature = "backend-lofty", not(feature = "backend-taglib")))]
static FILE_SET_TAGS: LazyLock<FileSetTags> = LazyLock::new(|| file_set_tags_lofty);

// ---- public entry points ----

/// Returns true if `path` is a valid audio file.
///
/// Backend is chosen at runtime: `TagLib` by default, `Lofty` with `-l`.
pub fn is_audiofile(path: &Path) -> bool {
    IS_AUDIOFILE(path)
}

/// Sets tags to `dst` audio file, composing the title from `ii` and `src`.
///
/// Backend is chosen at runtime: `TagLib` by default, `Lofty` with `-l`.
pub fn file_set_tags(ii: u64, src: &Path, dst: &Path) {
    FILE_SET_TAGS(ii, src, dst)
}
