# Procrustes a.k.a. Damastes

Ever got frustrated with an audiobook like this one?

```
Robinson Crusoe $ ls
'Disc 1'   'Disc 14'  'Disc 3'  'Disc 8'
'Disc 10'  'Disc 15'  'Disc 4'  'Disc 9'
'Disc 11'  'Disc 16'  'Disc 5'
'Disc 12'  'Disc 17'  'Disc 6'
'Disc 13'  'Disc 2'   'Disc 7'
```

```
Robinson Crusoe $ tree
...
├── Disc 17
│   ├── 01 Track 1.mp3
│   ├── 02 Track 2.mp3
...
│   ├── 13 Track 13.mp3
├── Disc 2
│   ├── 01 Track 1.mp3
│   ├── 02 Track 2.mp3
│   ├── 03 Track 3.mp3
...
│   ├── 15 Track 15.mp3
│   └── desktop.ini
├── Disc 3
│   ├── 01 Track 1.mp3
│   ├── 02 Track 2.mp3
...
```

Try **Procrustes**, this way:

```
Robinson Crusoe $ procrustes -via 'Daniel Defoe' -m 'Robinson Crusoe' . ~/MyAudioLibrary
```

- `MyAudioLibrary` must exist

or just like this:

```
Robinson Crusoe $ procrustes -a 'Daniel Defoe' -u 'Robinson Crusoe' . ~/MyAudioLibrary
```

Notice the tags set by **Procrustes**.

## Description

**Procrustes** is a CLI utility for basic processing and copying
of audio albums, mostly slovenly built audiobooks, possibly to cheap mobile
devices. Common poor design problems: track number tags
missing or incorrect, directory and/or file names enumerated
without leading zeroes, etc.

Meanwhile, one cannot listen to an audiobook with the tracks in the wrong
order. **Procrustes** tries hard to sort the tracks properly.
To check the track order visually use `-v` or `-vi`, and avoid `-u`.

**Procrustes** renames directories and audio files, replacing tags,
if necessary, while copying the album to destination. Source files
and directories are not modified in any way. Files are copied sequentially,
by default file number one first, optionally in reverse order, as some
mobile devices are copy-order sensitive.

## General syntax

```
$ procrustes [<options>] <source directory> <destination directory>
```

## Options

| Option                      | Description                                                                                                               |
| --------------------------- | ------------------------------------------------------------------------------------------------------------------------- |
| `-h, --help`                | _short description and options_                                                                                           |
| `-v, --verbose`             | _unless verbose, just progress bar is shown_                                                                              |
| `-d, --drop-tracknumber`    | _do not set track numbers_                                                                                                |
| `-s, --strip-decorations`   | _strip file and directory name decorations_                                                                               |
| `-f, --file-title`          | _use file name for title tag_                                                                                             |
| `-F, --file-title-num`      | _use numbered file name for title tag_                                                                                    |
| `-x, --sort-lex`            | _sort files lexicographically_                                                                                            |
| `-t, --tree-dst`            | _retain the tree structure of the source album at destination_                                                            |
| `-p, --drop-dst`            | _do not create destination directory_                                                                                     |
| `-r, --reverse`             | _copy files in reverse order (number one file is the last to be copied)_                                                  |
| `-w, --overwrite`           | _silently remove existing destination directory (not recommended)_                                                        |
| `-y, --dry-run`             | _without actually modifying anything (trumps_ `-w`, _too)_                                                                |
| `-c, --count`               | _just count the files_                                                                                                    |
| `-i, --prepend-subdir-name` | _prepend current subdirectory name to a file name_                                                                        |
| `-e, --file-type TEXT`      | _accept only audio files of the specified type, e.g._ `-e flac`, `-e '*64kb.mp3'`                                         |
| `-u, --unified-name TEXT`   | _destination root directory name and file names are based on_ `TEXT`, _serial number prepended, file extensions retained_ |
| `-a, --artist TEXT`         | _artist tag_                                                                                                              |
| `-m, --album TEXT`          | _album tag_                                                                                                               |
| `-b, --album-num INTEGER`   | _0..99; prepend_ `INTEGER` _to the destination root directory name_                                                       |

## Examples

```
Source Album $ procrustes -c . .
```

- All the files in _Source Album_ get checked. Destination directory is required (and ignored).

```
Source Album $ procrustes -y . .
```

- Dry run: everything is done according to any options; no new files or directories created, destination is left undisturbed.

```
Source Album $ procrustes -a "Peter Crowcroft" -m "Mice All Over" . /run/media/user/F8950/Audiobooks/
```

- Destination directory _/run/media/user/F8950/Audiobooks/Source Album/_ is created;

- Track numbers are set according to the natural sort order, regardless of the absence of the original leading zeroes:

```
01-mice-all-over-1.mp3
02-mice-all-over-2.mp3
...
09-mice-all-over-9.mp3
10-mice-all-over-10.mp3
11-mice-all-over-11.mp3
...
```

- _Artist_ is set to _Peter Crowcroft_;

- _Album_ is set to _Mice All Over_;

- _Title_ is set to _1 P.C. - Mice All Over_ for the first file, all titles enumerated;

```
Source Album $ procrustes -dst . /run/media/user/F8950/Audiobooks/
```

- _Source Album_ directory is copied to _/run/media/user/F8950/Audiobooks/_ in its entirety, without modification; sequential copy order, natural or lexicographical, is guaranteed.
