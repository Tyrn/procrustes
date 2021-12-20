Procrustes a.k.a. Damastes
**************************

Ever got frustrated with an audiobook like this one?

::

    Robinson Crusoe $ ls
    'Disc 1'   'Disc 14'  'Disc 3'  'Disc 8'
    'Disc 10'  'Disc 15'  'Disc 4'  'Disc 9'
    'Disc 11'  'Disc 16'  'Disc 5'
    'Disc 12'  'Disc 17'  'Disc 6'
    'Disc 13'  'Disc 2'   'Disc 7'

::

    Robinson Crusoe $ tree
    ...
    ├── Disc 17
    │   ├── 01 Track 1.mp3
    │   ├── 02 Track 2.mp3
    ...
    │   ├── 13 Track 13.mp3
    ├── Disc 2
    │   ├── 01 Track 1.mp3
    │   ├── 02 Track 2.mp3
    │   ├── 03 Track 3.mp3
    ...
    │   ├── 15 Track 15.mp3
    │   └── desktop.ini
    ├── Disc 3
    │   ├── 01 Track 1.mp3
    │   ├── 02 Track 2.mp3
    ...

Try **Procrustes**, this way:

::

    Robinson Crusoe $ procrustes -via 'Daniel Defoe' -m 'Robinson Crusoe' . ~/MyAudioLibrary

- ``MyAudioLibrary`` must exist

or just like this:

::

    Robinson Crusoe $ procrustes -a 'Daniel Defoe' -u 'Robinson Crusoe' . ~/MyAudioLibrary

Notice the tags set by **Procrustes**.

Description
===========

**Procrustes** is a CLI utility for basic processing and copying
of audio albums, mostly slovenly built audiobooks, possibly to cheap mobile
devices. Common poor design problems: track number tags
missing or incorrect, directory and/or file names enumerated
without leading zeroes, etc.

Meanwhile, one cannot listen to an audiobook with the tracks in the wrong
order. **Procrustes** tries hard to sort the tracks properly.
To check the track order visually use ``-v`` or ``-vi``, and avoid ``-u``.

**Procrustes** renames directories and audio files, replacing tags,
if necessary, while copying the album to destination. Source files
and directories are not modified in any way. Files are copied sequentially,
by default file number one first, optionally in reverse order, as some
mobile devices are copy-order sensitive.

General syntax
==============

::

    $ procrustes [<options>] <source directory> <destination directory>

Options
=======

``-h, --help``                       *short description and options*

``-V, --version``                    *package version*

``-v, --verbose``                    *unless verbose, just progress bar is shown*

``-d, --drop-tracknumber``           *do not set track numbers*

``-s, --strip-decorations``          *strip file and directory name decorations*

``-f, --file-title``                 *use file name for title tag*

``-F, --file-title-num``             *use numbered file name for title tag*

``-x, --sort-lex``                   *sort files lexicographically*

``-t, --tree-dst``                   *retain the tree structure of the source album at destination*

``-p, --drop-dst``                   *do not create destination directory*

``-r, --reverse``                    *copy files in reverse order (number one file is the last to be copied)*

``-w, --overwrite``                  *silently remove existing destination directory (not recommended)*

``-y, --dry-run``                    *without actually modifying anything (trumps* ``-w``, *too)*

``-c, --count``                      *just count the files*

``-i, --prepend-subdir-name``        *prepend current subdirectory name to a file name*

``-e, --file-type TEXT``             *accept only audio files of the specified type, e.g.* ``-e flac``, ``-e '*64kb.mp3'``

``-u, --unified-name TEXT``          *destination root directory name and file names are based on* ``TEXT``, *serial number prepended, file extensions retained*

``-a, --artist TEXT``                *artist tag*

``-m, --album TEXT``                 *album tag*

``-b, --album-num INTEGER``          *0..99; prepend* ``INTEGER`` *to the destination root directory name*

Examples
========

::

    Source Album $ procrustes -c . .

- All the files in *Source Album* get checked. Destination directory is required (and ignored).

::

    Source Album $ procrustes -y . .

- Dry run: everything is done according to any options; no new files or directories created, destination is left undisturbed.

::

    Source Album $ procrustes -a "Peter Crowcroft" -m "Mice All Over" . /run/media/user/F8950/Audiobooks/

- Destination directory */run/media/user/F8950/Audiobooks/Source Album/* is created;

- Track numbers are set according to the natural sort order, regardless of the absence of the original leading zeroes:

::

    01-mice-all-over-1.mp3
    02-mice-all-over-2.mp3
    ...
    09-mice-all-over-9.mp3
    10-mice-all-over-10.mp3
    11-mice-all-over-11.mp3
    ...

- *Artist* is set to *Peter Crowcroft*;

- *Album* is set to *Mice All Over*;

- *Title* is set to *1 P.C. - Mice All Over* for the first file, all titles enumerated;

::

    Source Album $ procrustes -dst . /run/media/user/F8950/Audiobooks/

- *Source Album* directory is copied to */run/media/user/F8950/Audiobooks/* in its entirety, without modification; sequential copy order, natural or lexicographical, is guaranteed.
