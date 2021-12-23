Procrustes a.k.a. Damastes
**************************

Audio album builder; copy and edit on the fly.

Development
===========

- `Format Rust code <https://github.com/rust-lang/rustfmt>`__
- `TagLib <https://github.com/taglib/taglib>`__ library, release build, is required on the system. For the Archlinux family available on AUR:

::

    $ yay -S taglib-git

Build
-----

::

    $ cargo build [--release]

Format
------

::

    $ cargo fmt

Test
----

::

    $ cargo test

Install
-------

Install to ``~/.cargo/bin``:

::

    $ cargo install --path . [--debug]

Publish
-------

::

    TODO
