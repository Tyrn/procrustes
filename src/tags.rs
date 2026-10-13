//! Audio file detection and tag editing.
//!
//! Two backends are available, selected at compile time:
//!
//!   * `backend-taglib` — uses the TagLib C++ library (default)
//!   * `backend-lofty`  — uses the pure-Rust `lofty` crate
//!
//! Exactly one must be enabled. The public functions
//! [`is_audiofile`] and [`file_set_tags`] have identical signatures
//! under either backend, so call sites need no change.
//!
//! Build selection:
//!
//! ```text
//! cargo build                                          # taglib (default)
//! cargo build --no-default-features --features backend-lofty
//! ```

use crate::{flag, VERBOSE_LEVEL};
use std::path::Path;
use std::sync::LazyLock;

// ---------------------------------------------------------------------
// Backend selection guards
// ---------------------------------------------------------------------

#[cfg(all(feature = "backend-taglib", feature = "backend-lofty"))]
compile_error!("features `backend-taglib` and `backend-lofty` are mutually exclusive");

#[cfg(not(any(feature = "backend-taglib", feature = "backend-lofty")))]
compile_error!("exactly one of `backend-taglib` or `backend-lofty` must be enabled");

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
// These are compiled under either backend, so both share one copy.
// ---------------------------------------------------------------------

use crate::{initials, ALBUM, ARTIST, IS_ALBUM, IS_ARTIST};
// (adjust the import path above to match where these live in your crate)

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
/// Returns true, if [path] is a valid audio file, otherwise false.
///
pub fn is_audiofile(path: &Path) -> bool {
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
/// Sets tags to [dst] audio file, using [ii] and [src] name in the title tag
/// composition.
///
pub fn file_set_tags(ii: u64, src: &Path, dst: &Path) {
    use crate::{flag, BDELIM_ICON};

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

#[cfg(feature = "backend-lofty")]
pub fn is_audiofile(path: &Path) -> bool {
    use crate::is_pattern_ok;

    let _silence = quiet_if_not_verbose();

    if !is_pattern_ok(path) {
        return false;
    }
    lofty::read_from_path(path).is_ok()
}

#[cfg(feature = "backend-lofty")]
pub fn file_set_tags(ii: u64, src: &Path, dst: &Path) {
    use crate::{flag, BDELIM_ICON};

    use lofty::config::WriteOptions;
    use lofty::file::AudioFile;
    use lofty::prelude::{Accessor, TagExt, TaggedFileExt};
    use lofty::tag::Tag;

    let _silence = quiet_if_not_verbose();

    // ---- open the destination ----

    let mut tagged = lofty::read_from_path(dst).unwrap_or_else(|_| {
        panic!(
            "{}Error while opening \"{}\" for tagging.{}",
            BDELIM_ICON,
            dst.to_str().unwrap(),
            BDELIM_ICON
        )
    });

    // Choose a tag type appropriate for the file's format, creating an
    // empty tag if the file has none yet.
    let tag_type = tagged.primary_tag_type();
    if tagged.primary_tag_mut().is_none() {
        tagged.insert_tag(Tag::new(tag_type));
    }

    let tag = tagged
        .primary_tag_mut()
        .unwrap_or_else(|| panic!("{}No tagging data.{}", BDELIM_ICON, BDELIM_ICON));

    // ---- compose and apply ----

    tag.set_title(TITLE_COMPOSE(ii, src));

    if !flag("d") {
        tag.set_track(ii as u32);
    }

    if *IS_ARTIST {
        tag.set_artist(ARTIST.clone());
    }
    if *IS_ALBUM {
        tag.set_album(ALBUM.clone());
    }

    // ---- write back ----

    tagged
        .save_to_path(dst, WriteOptions::default())
        .unwrap_or_else(|_| {
            panic!(
                "{}Error while saving tags to \"{}\".{}",
                BDELIM_ICON,
                dst.to_str().unwrap(),
                BDELIM_ICON
            )
        });
}
