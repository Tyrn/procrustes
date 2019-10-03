## Procrustes SmArT

### Description

**Procrustes SmArT** is a CLI utility for basic processing and copying
of audio albums, mostly audiobooks of uncertain provenance to cheap mobile
devices. Audiobooks in question can be poorly designed: track number tags
may be missing or incorrect, directory and/or file names enumerated
without leading zeroes, etc.

**Procrustes SmArT** renames directories and audio files, replacing tags,
if necessary, while copying the album to destination. Source files
and directories are not modified in any way. Files are copied sequentially,
by default file number one first, optionally in reverse order, as some
mobile devices are copy-order sensitive.

### General syntax


``$ procrustes [<options>] <source directory> <destination directory>``

### Options

``-h, --help``
short description and options

``-v, --verbose``
unless verbose, just progress bar is shown

``-d, --drop-tracknumber``
do not set track numbers

``-s, --strip-decorations``
strip file and directory name decorations

``-f, --file-title``
use file name for title tag

``-F, --file-title-num``
use numbered file name for title tag

``-x, --sort-lex``
sort files lexicographically

``-t, --tree-dst``
retain the tree structure of the source album at destination

``-p, --drop-dst``
do not create destination directory

``-r, --reverse``
copy files in reverse order (number one file is the last to be copied)

``-e, --file-type FILE_TYPE``
accept only audio files of the specified type

``-i, --prepend-subdir-name``
prepend current subdirectory name to a file name

``-u, --unified-name UNIFIED_NAME``
destination root directory name and file names are based on UNIFIED_NAME,
serial nuber prepended, file extentions retained

``-b, --album-num ALBUM_NUM``
0..99; prepend ALBUM_NUM to the destination root directory name

``-a --artist-tag ARTIST_TAG``
artist tag name

``-g --album-tag ALBUM_TAG``
album tag name

### Examples
```
Source Album $ procrustes -a "Peter Crowcroft" -g "Mice All Over" . /run/media/user/F8950/Audiobooks/
```
- Destination directory `/run/media/user/F8950/Audiobooks/Source Album/` is created;

- Track numbers are set according to the natural sort order (file names `..., 5, 6, 7, 8, 9, 10...`;
regardless of the absence of the leading zeroes);

- *Artist* is set to *Peter Crowcroft*;

- *Album* is set to *Mice All Over*;

- *Title* is set to *1 P.C. - Mice All Over* for the first file, all titles enumerated;
```
Source Album $ procrustes -dst . /run/media/user/F8950/Audiobooks/
```
- *Source Album* directory is copied to `/run/media/user/F8950/Audiobooks/` in its entirety,
without modification; sequential copy order, natural or lexicographical, is guaranteed.
